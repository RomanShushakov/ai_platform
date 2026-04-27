#!/usr/bin/env bash
set -euo pipefail

REGISTRY=192.168.178.103:5000
IMAGE=$REGISTRY/ai-platform-host:latest

docker buildx build \
  --platform linux/arm64 \
  -f host/dockerfile \
  -t "$IMAGE" \
  --push .
