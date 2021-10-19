#!/bin/bash
set -xeuf -o pipefail

sudo apt update
sudo apt install -y build-essential libssl-dev cmake freetype2-demos libfreetype-dev
