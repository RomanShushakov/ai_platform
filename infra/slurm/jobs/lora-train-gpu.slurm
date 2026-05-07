#!/usr/bin/env bash
#SBATCH --job-name=lora-train-gpu
#SBATCH --output=/home/roman/nfs/lora/logs/lora-train-gpu-%j.out
#SBATCH --error=/home/roman/nfs/lora/logs/lora-train-gpu-%j.err
#SBATCH --nodes=1
#SBATCH --nodelist=jetson
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=4G
#SBATCH --gres=gpu:1
#SBATCH --time=02:00:00

set -euo pipefail

SIF="/home/roman/nfs/lora/images/lora-trainer-cuda.sif"
TRAIN_SCRIPT="/home/roman/workdir/ai_platform/lora/training/train_qwen_lora_gpu.py"

export BASE_MODEL="/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct"
export DATASET_PATH="/home/roman/nfs/lora/datasets/lab_style.jsonl"
export OUTPUT_DIR="/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_${SLURM_JOB_ID}"
export MAX_STEPS="20"
export NUM_TRAIN_EPOCHS="10"
export MAX_LENGTH="64"
export LEARNING_RATE="2e-4"
export BATCH_SIZE="1"
export GRAD_ACCUM_STEPS="1"
export PYTORCH_NO_CUDA_MEMORY_CACHING="1"
export CUDA_MODULE_LOADING="LAZY"

mkdir -p /home/roman/nfs/lora/logs
mkdir -p /home/roman/nfs/lora/adapters

echo "Job ID: ${SLURM_JOB_ID}"
echo "Node: $(hostname)"
echo "SLURM CUDA_VISIBLE_DEVICES=${CUDA_VISIBLE_DEVICES:-unset}"
echo "OUTPUT_DIR=${OUTPUT_DIR}"

unset CUDA_VISIBLE_DEVICES

echo "Container CUDA check"

sudo -n apptainer exec \
  --nv \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  --bind /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu:ro \
  --bind /usr/local/cuda:/usr/local/cuda:ro \
  --env LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu/nvidia:/usr/local/cuda/lib64 \
  "${SIF}" \
  python3 -c "import torch, transformers; print(torch.__version__); print(transformers.__version__); print(torch.version.cuda); print(torch.cuda.is_available()); print(torch.cuda.device_count())"

echo "Starting GPU LoRA training"

sudo -n apptainer exec \
  --nv \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  --bind /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu:ro \
  --bind /usr/local/cuda:/usr/local/cuda:ro \
  --env BASE_MODEL="${BASE_MODEL}" \
  --env DATASET_PATH="${DATASET_PATH}" \
  --env OUTPUT_DIR="${OUTPUT_DIR}" \
  --env MAX_STEPS="${MAX_STEPS}" \
  --env NUM_TRAIN_EPOCHS="${NUM_TRAIN_EPOCHS}" \
  --env MAX_LENGTH="${MAX_LENGTH}" \
  --env LEARNING_RATE="${LEARNING_RATE}" \
  --env BATCH_SIZE="${BATCH_SIZE}" \
  --env GRAD_ACCUM_STEPS="${GRAD_ACCUM_STEPS}" \
  --env LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu/nvidia:/usr/local/cuda/lib64 \
  --env PYTORCH_NO_CUDA_MEMORY_CACHING="${PYTORCH_NO_CUDA_MEMORY_CACHING}" \
  --env CUDA_MODULE_LOADING="${CUDA_MODULE_LOADING}" \
  "${SIF}" \
  python3 "${TRAIN_SCRIPT}"

echo "GPU adapter written to ${OUTPUT_DIR}"
