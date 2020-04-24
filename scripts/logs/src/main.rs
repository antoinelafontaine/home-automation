use recap::Recap;
use serde::Deserialize;
use std::error::Error;
use easy_reader::EasyReader;
use std::fs::File;

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

#[derive(Deserialize, Debug)]
struct Env {
    discord_mineshaft_webhhook: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let minecraft_logs = "/opt/minecraft/server/shared/logs/latest.log";
    let discord_webhook;
    let last_log_entry;
    let mut reader;

    match envy::from_env::<Env>() {
        Err(e) => return Err(From::from(e)),
        Ok(env) => {
            discord_webhook = env.discord_mineshaft_webhhook;
        },
    }

    // Find log last entry
    match File::open(minecraft_logs) {
        Err(e) => return Err(From::from(e)),
        Ok(logs) => {
            reader = EasyReader::new(logs)?;
            reader.eof();
            last_log_entry = reader.prev_line()?.unwrap();
        },
    };

    // Rewind to last log entry
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
                    },
                    _ => (),
                } 
            }
        }        
    }
    
    Ok(())
}
