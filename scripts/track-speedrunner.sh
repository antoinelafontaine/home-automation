#!/bin/bash

mcrcon "say Starting the hunt for $1"
while true; do
  sleep 300
  mcrcon "say 5 minutes elapsed"
  sleep 240
  mcrcon "say 1 minute left before reveiling location!"
  sleep 60
  mcrcon "say $(mcrcon 'data get entity Neverprint Pos')"
done
