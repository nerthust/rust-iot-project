#!/bin/bash
set -xeuf -o pipefail

sudo apt update
sudo apt install -y build-essential libssl-dev cmake freetype2-demos \
                    libfreetype-dev libglib2.0-dev libcairo2-dev \
                    libsdl-pango-dev libgtk-3-dev libcanberra-gtk-module \
                    libcanberra-gtk3-module
