#!/bin/bash
export CC=gcc
export CXX=g++
export AR=ar
export PATH="/home/samuel/.cargo/bin:$PATH"
cd '/mnt/c/Users/smateorodriguez/OneDrive - Deloitte (O365D)/Documents/personal-projects/egui_graphs/crates/code-analyzer-web'
exec /home/samuel/.cargo/bin/trunk serve --release
