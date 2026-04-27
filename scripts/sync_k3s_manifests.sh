#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=raspberry
REMOTE_DIR=/home/roman/workdir/k3s/manifests/ai_platform
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/raspberry

LOCAL_DIR=infra/k3s/manifests/ai_platform/

echo "Syncing manifests to $REMOTE_HOST..."

ssh -i ${SSH_PRIVATE_KEY_FILE} ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ${REMOTE_DIR}"

rsync -avz --delete \
  ${LOCAL_DIR} \
  ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/

echo "Done."
