#!/usr/bin/env bash

set -euo pipefail

systemctl --user stop void-backend
systemctl --user stop void-frontend
systemctl --user stop void-murmur
