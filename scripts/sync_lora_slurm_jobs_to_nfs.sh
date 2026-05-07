#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=raspberry
REMOTE_DIR=/home/roman/nfs/lora/jobs
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/raspberry

LOCAL_DIR=infra/slurm/jobs

echo "Syncing LoRA Slurm jobs to $REMOTE_HOST..."

ssh -i ${SSH_PRIVATE_KEY_FILE} ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ${REMOTE_DIR}"

rsync -avz \
  -e "ssh -i ${SSH_PRIVATE_KEY_FILE}" \
  "${LOCAL_DIR}/lora-train.slurm" \
  "${LOCAL_DIR}/lora-train-gpu.slurm" \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/"

echo "Done."
