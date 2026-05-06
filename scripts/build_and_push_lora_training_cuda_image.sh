#!/usr/bin/env bash
set -euo pipefail

REGISTRY=192.168.178.103:5000
IMAGE=$REGISTRY/lora-trainer-cuda:latest

docker buildx build \
  --platform linux/arm64 \
  -f infra/images/lora_training_cuda/dockerfile \
  -t "$IMAGE" \
  --push .
