#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=raspberry
REMOTE_DIR=/home/roman/nfs/lora
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/raspberry

echo "Creating LoRA directories on NFS..."

ssh -i ${SSH_PRIVATE_KEY_FILE} ${REMOTE_USER}@${REMOTE_HOST} "
  mkdir -p '${REMOTE_DIR}/datasets'
  mkdir -p '${REMOTE_DIR}/adapters'
  mkdir -p '${REMOTE_DIR}/scripts'
  mkdir -p '${REMOTE_DIR}/jobs/logs'
  mkdir -p '${REMOTE_DIR}/jobs/work'
"

echo "Syncing LoRA datasets..."

rsync -avz \
  lora/datasets/ \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/datasets/"

echo "Done."
