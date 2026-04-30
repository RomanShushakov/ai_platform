#!/usr/bin/env bash
set -euo pipefail

REGISTRY=192.168.178.103:5000
IMAGE=$REGISTRY/rag-indexer:latest

docker buildx build \
  --platform linux/arm64 \
  -f rag_indexer/dockerfile \
  -t "$IMAGE" \
  --push .
