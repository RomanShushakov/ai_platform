#!/usr/bin/env bash
set -euo pipefail

IMAGE="docker://192.168.178.103:5000/lora-trainer:latest"
SIF_DIR="/home/roman/nfs/lora/images"
SIF_PATH="${SIF_DIR}/lora-trainer_latest.sif"

mkdir -p "${SIF_DIR}"

apptainer build \
  "${SIF_PATH}" \
  "${IMAGE}"

echo "Built ${SIF_PATH}"
