// Minecraft logs bot in Rust

extern crate reqwest;
extern crate notify;
extern crate daemonize;
extern crate envy;
extern crate recap;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::error::Error;
use std::time::Duration;
use std::thread;
use std::sync::mpsc::channel;

use notify::{Watcher, RecursiveMode, watcher};
use daemonize::Daemonize;
use recap::Recap;
use serde::Deserialize;
use easy_reader::EasyReader;

// Environement variable parser
#[derive(Debug, Deserialize)]
struct Env {
    discord_mineshaft_webhook: String,
}

// Minecraft log struct and parser
#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"(?x)
        \[
        (?P<timestamp>[0-9]+:[0-9]+:[0-9]+)
        \]\s\[
        (?P<meta>.+)
        \]:\s
        (?P<message>.+$)
    "#)]
struct MinecraftLog {
    timestamp: String,
    meta: String,
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Make sure we have the webhook set before starting our daemon
    let discord_webhook;

    match envy::from_env::<Env>() {
        Err(e) => return Err(From::from(e)),
        Ok(env) => {
            discord_webhook = env.discord_mineshaft_webhook;
        },
    }
    
    let stdout = File::create("/tmp/mineshaft-bot.out").unwrap();
    let stderr = File::create("/tmp/mineshaft-bot.err").unwrap();
    let daemonize = Daemonize::new()
        .pid_file("/tmp/mineshaft-bot.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .user("nobody")
        .group("daemon")
        .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Err(e) => println!("daemonize error: {:?}", e),
        Ok(_) => {
            let minecraft_logs = "/opt/minecraft/server/shared/logs/latest.log";
            let mut last_log_entry;
            let mut reader;

            println!("Starting Mineshaft Bot daemon");

            // Find log last entry
            match File::open(minecraft_logs) {
                Err(e) => return Err(From::from(e)),
                Ok(logs) => {
                    reader = EasyReader::new(logs)?;
                    reader.eof();
                    last_log_entry = reader.prev_line()?.unwrap();
                },
            };

            println!("Last log entry when bot was started: {}", last_log_entry);

            let (tx, rx) = channel();
            let mut watcher = watcher(tx, Duration::from_millis(300)).unwrap();
            let client = reqwest::blocking::Client::new();

            watcher.watch(minecraft_logs, RecursiveMode::Recursive).unwrap();

            loop {
                let event = rx.recv()?;
                
                match event {
                    notify::DebouncedEvent::Write(_) |
                    notify::DebouncedEvent::Create(_) |
                    notify::DebouncedEvent::Remove(_) => {
                        
                        // Rewind to last log entry
                        let logs = File::open(minecraft_logs)?;

                        reader = EasyReader::new(logs)?;
                        reader.eof();
                        loop {
                            match reader.prev_line()? {
                                None => break,
                                Some(line) => {
                                    if line == last_log_entry {
                                        break;
                                    }
                                },
                            }
                        }

                        loop {
                            match reader.next_line()? {
                                None => break,
                                Some(line) => {
                                    match line.parse::<MinecraftLog>() {
                                        Ok(entry) => {
                                            println!("{:#?}", entry);
                                            match check_message(entry.message) {
                                                Ok(message) => {
                                                    let response = client.post(&discord_webhook)
                                                    .json(&serde_json::json!({
                                                        "content": message,
                                                    })).send()?;
                                                    println!("{:#?}", response);
                                                },
                                                _ => ()
                                            }
                                            last_log_entry = line;
                                        },
                                        // If there is an error parsing the line, skip it.
                                        _ => (),
                                    } 
                                }
                            } 
                            // wait before next post to not overload the Discord api
                            thread::sleep(Duration::from_millis(300));       
                        }
                    },

                    // do nothing on events other than Write.
                    _ => (),
                }
            }
        },
    }

    Ok(())
}

#[derive(Debug)]
pub enum MessageError {
    MessageDiscarded,
    UnknownMessage,
}

fn check_message(message: String) -> Result<String, MessageError> {
    if message.contains("logged in with entity") ||
       message.contains("lost connection: Disconnected") ||
       message.contains("UUID of player") {
        return Err(MessageError::MessageDiscarded);
    }

    if message.contains("joined the game") {
        return Ok(format!("```css\n- {}```", message));
    }
    if message.contains("left the game") {
        return Ok(format!("```diff\n- {}```", message));
    }
        
    Ok(message)
}