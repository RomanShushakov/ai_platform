#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=jetson
REMOTE_DIR=/home/roman/workdir/rag_indexer_jetson
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/jetson

LOCAL_DIR=rag_indexer_jetson/

echo "Syncing rag_indexer_jetson to ${REMOTE_HOST}:${REMOTE_DIR}..."

ssh -i "${SSH_PRIVATE_KEY_FILE}" "${REMOTE_USER}@${REMOTE_HOST}" \
  "mkdir -p '${REMOTE_DIR}'"

rsync -avz --delete \
  -e "ssh -i ${SSH_PRIVATE_KEY_FILE}" \
  "${LOCAL_DIR}" \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/"

echo "Done."
