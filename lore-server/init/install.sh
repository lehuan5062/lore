#!/usr/bin/env bash

set -x

sudo systemctl stop lore

sudo mkdir -p /etc/lore/config

sudo cp loreserver /usr/local/bin
sudo cp -R config/ /etc/lore/
sudo chmod +x /usr/local/bin/loreserver

sudo cp lore.service /etc/systemd/system
sudo systemctl daemon-reload

sudo systemctl start lore
