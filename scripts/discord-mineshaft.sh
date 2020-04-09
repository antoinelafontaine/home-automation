#!/bin/bash

if [ -z "${DISCORD_MINESHAFT_WEBHHOOK}" ]; then
    echo "Missing Environement variable: DISCORD_MINESHAFT_WEBHHOOK"
else
    # discord webhook
    webhook="${DISCORD_MINESHAFT_WEBHHOOK}"
    latest_timestamp=$(tail -n 1 /opt/minecraft/server/shared/logs/latest.log | grep -oP "[0-9]+:[0-9]+:[0-9]+")
    messages=$(tail -n 10 /opt/minecraft/server/shared/logs/latest.log | grep -P "$latest_timestamp")

    while IFS= read -r line
    do
        echo "$line"
        curl -H "Content-Type: application/json" -X POST -d "{\"content\": \"$line\"}" $webhook
        sleep 0.750
    done < <(printf '%s\n' "$messages")
fi
