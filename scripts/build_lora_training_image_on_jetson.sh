#!/usr/bin/env bash
set -euo pipefail

IMAGE="192.168.178.103:5000/lora-trainer:latest"

cd ~/workdir/ai_platform

docker build \
  -f infra/images/lora_training/dockerfile \
  -t "${IMAGE}" \
  .

docker push "${IMAGE}"

echo "Pushed ${IMAGE}"
