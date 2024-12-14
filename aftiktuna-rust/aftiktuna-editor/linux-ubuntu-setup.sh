#!/bin/bash

sudo apt-get update
# Dependencies needed by "macroquad"
sudo apt-get install pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
# Dependency needed by the GTK backend of "rfd"
sudo apt install libgtk-3-dev
