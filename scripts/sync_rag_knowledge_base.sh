#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=raspberry
REMOTE_DIR=/home/roman/nfs/rag/knowledge_base
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/raspberry

LOCAL_DIR=knowledge_base/

echo "Syncing knowledge_base to ${REMOTE_HOST}:${REMOTE_DIR}..."

ssh -i "${SSH_PRIVATE_KEY_FILE}" "${REMOTE_USER}@${REMOTE_HOST}" \
  "mkdir -p '${REMOTE_DIR}'"

rsync -avz --delete \
  -e "ssh -i ${SSH_PRIVATE_KEY_FILE}" \
  "${LOCAL_DIR}" \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/"

echo "Done."
