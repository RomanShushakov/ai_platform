#!/usr/bin/env bash
#SBATCH --job-name=gpu-smoke
#SBATCH --output=/home/roman/nfs/lora/logs/gpu-smoke-%j.out
#SBATCH --error=/home/roman/nfs/lora/logs/gpu-smoke-%j.err
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=2G
#SBATCH --gres=gpu:1
#SBATCH --time=00:05:00

set -euo pipefail

echo "Job ID: ${SLURM_JOB_ID}"
echo "Node: $(hostname)"
echo "CUDA_VISIBLE_DEVICES=${CUDA_VISIBLE_DEVICES:-unset}"

ls -l /dev/nvhost* 2>/dev/null | head || true
ls -l /dev/nvidia* 2>/dev/null || true

echo "GPU smoke job completed"
