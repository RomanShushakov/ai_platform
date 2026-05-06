#!/usr/bin/env bash
#SBATCH --job-name=lora-train
#SBATCH --output=/home/roman/nfs/lora/logs/lora-train-%j.out
#SBATCH --error=/home/roman/nfs/lora/logs/lora-train-%j.err
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=6G
#SBATCH --time=02:00:00

set -euo pipefail

REPO_DIR="/home/roman/nfs/repos/ai_platform"
SIF="/home/roman/nfs/lora/images/lora-trainer_latest.sif"

BASE_MODEL="/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct"
DATASET="/home/roman/nfs/lora/datasets/lab_style.jsonl"
OUTPUT_DIR="/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_${SLURM_JOB_ID}"

mkdir -p /home/roman/nfs/lora/logs
mkdir -p /home/roman/nfs/lora/adapters

echo "Job ID: ${SLURM_JOB_ID}"
echo "Node: $(hostname)"
echo "Base model: ${BASE_MODEL}"
echo "Dataset: ${DATASET}"
echo "Output: ${OUTPUT_DIR}"

apptainer exec \
  --bind /home/roman/nfs:/home/roman/nfs \
  "${SIF}" \
  python "${REPO_DIR}/lora/training/train_qwen_lora.py" \
    --base-model "${BASE_MODEL}" \
    --dataset "${DATASET}" \
    --output-dir "${OUTPUT_DIR}" \
    --max-steps 50 \
    --learning-rate 2e-4 \
    --batch-size 1 \
    --gradient-accumulation-steps 1

echo "Adapter written to ${OUTPUT_DIR}"
