#!/bin/bash
#SBATCH --job-name=gpu-probe-gres
#SBATCH --output=/home/roman/workdir/slurm/job-examples/slurm-%j.out
#SBATCH --partition=debug
#SBATCH --gres=gpu:1

set -x

echo "=== hostname ==="
hostname

echo "=== date ==="
date

echo "=== visible NVIDIA devices ==="
ls -l /dev/nvidia0 /dev/nvidiactl /dev/nvidia-modeset 2>/dev/null || true

echo "=== tegrastats ==="
which tegrastats || true
timeout 3 tegrastats || true

echo "Requested GPU via Slurm GRES"
