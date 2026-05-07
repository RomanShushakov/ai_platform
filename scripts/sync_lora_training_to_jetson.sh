#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=jetson
REMOTE_DIR=/home/roman/workdir/ai_platform/lora/training
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/jetson

LOCAL_DIR=lora/training

echo "Syncing LoRA training scripts to $REMOTE_HOST..."

ssh -i ${SSH_PRIVATE_KEY_FILE} ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ${REMOTE_DIR}"

rsync -avz \
  -e "ssh -i ${SSH_PRIVATE_KEY_FILE}" \
  "${LOCAL_DIR}/train_qwen_lora.py" \
  "${LOCAL_DIR}/train_qwen_lora_gpu.py" \
  "${LOCAL_DIR}/requirements.txt" \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/"

echo "Done."
