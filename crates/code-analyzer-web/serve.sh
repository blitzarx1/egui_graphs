#!/bin/bash
cd "$(dirname "$0")"
source ~/.cargo/env
export CARGO_TARGET_DIR=~/egui_graphs_target
trunk serve --address 127.0.0.1 --port 8080
