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

WORKDIR="/home/roman/workdir/ai_platform"
SIF="/home/roman/nfs/lora/images/lora-trainer.sif"

export BASE_MODEL="/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct"
export DATASET_PATH="/home/roman/nfs/lora/datasets/lab_style.jsonl"
export OUTPUT_DIR="/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_${SLURM_JOB_ID}"
export MAX_STEPS="50"
export NUM_TRAIN_EPOCHS="20"
export MAX_LENGTH="256"
export LEARNING_RATE="2e-4"
export BATCH_SIZE="1"
export GRAD_ACCUM_STEPS="1"

mkdir -p /home/roman/nfs/lora/logs
mkdir -p /home/roman/nfs/lora/adapters

echo "Job ID: ${SLURM_JOB_ID}"
echo "Node: $(hostname)"
echo "WORKDIR=${WORKDIR}"
echo "SIF=${SIF}"
echo "BASE_MODEL=${BASE_MODEL}"
echo "DATASET_PATH=${DATASET_PATH}"
echo "OUTPUT_DIR=${OUTPUT_DIR}"
echo "MAX_STEPS=${MAX_STEPS}"
echo "NUM_TRAIN_EPOCHS=${NUM_TRAIN_EPOCHS}"

apptainer exec \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  "${SIF}" \
  bash -lc "cd ${WORKDIR} && python lora/training/train_qwen_lora.py"

echo "Adapter written to ${OUTPUT_DIR}"
