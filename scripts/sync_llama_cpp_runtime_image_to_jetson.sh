#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=jetson
REMOTE_DIR=/home/roman/workdir/ai_platform
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/jetson

LOCAL_DIR=infra/images/llama_cpp_runtime/
REMOTE_IMAGE_DIR="${REMOTE_DIR}/infra/images/llama_cpp_runtime"

echo "Syncing llama.cpp runtime image files to ${REMOTE_HOST}:${REMOTE_IMAGE_DIR}..."

ssh -i "${SSH_PRIVATE_KEY_FILE}" "${REMOTE_USER}@${REMOTE_HOST}" \
  "mkdir -p '${REMOTE_IMAGE_DIR}'"

rsync -avz --delete \
  -e "ssh -i ${SSH_PRIVATE_KEY_FILE}" \
  "${LOCAL_DIR}" \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_IMAGE_DIR}/"

echo "Done."
