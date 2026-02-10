#!/usr/bin/env bash

set -euo pipefail

USER_SYSTEMD_CONTAINERS_DIR="$HOME/.config/containers/systemd"
USER_CONFIG_DIR="$HOME/.config/void-eid"
mkdir -p "$USER_SYSTEMD_CONTAINERS_DIR"
mkdir -p "$USER_CONFIG_DIR"

cp *.container "$USER_SYSTEMD_CONTAINERS_DIR"
cp *.network "$USER_SYSTEMD_CONTAINERS_DIR"
cp ../../.env "$USER_CONFIG_DIR"

systemctl --user daemon-reload
systemctl --user restart void-network
systemctl --user restart void-backend
systemctl --user restart void-frontend
systemctl --user restart void-murmur
