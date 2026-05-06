# 11. LoRA Training with Slurm and Apptainer

This document promotes the LoRA proof from local/manual training into a cluster-style workflow.

Current status:

- CPU LoRA training through Slurm + Apptainer works.
- Adapter output is saved to NFS.
- Adapter can be converted to GGUF with llama.cpp.
- GPU inference is already used by K3s llama.cpp through `runtimeClassName: nvidia` and `-ngl 99`.
- GPU LoRA training through Apptainer works as a CUDA visibility check from a normal Jetson shell.
- GPU LoRA training through Slurm is not complete yet.
- Slurm-launched Apptainer currently fails to expose CUDA correctly on Jetson.
- CUDA training image also needs package pinning because latest `transformers` is incompatible with the Jetson PyTorch image currently used.

## Paths

```text
Repo:
  ~/workdir/ai_platform

NFS root:
  /home/roman/nfs

LoRA dataset:
  /home/roman/nfs/lora/datasets/lab_style.jsonl

HF base model:
  /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct

CPU training SIF:
  /home/roman/nfs/lora/images/lora-trainer.sif

CUDA training SIF:
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif

Adapter output:
  /home/roman/nfs/lora/adapters

llama.cpp checkout:
  ~/workdir/llama.cpp
```

## 1. Create CPU LoRA training image

```bash
# on laptop
cd ~/workdir/ai_platform

mkdir -p infra/images/lora_training
mkdir -p infra/slurm/jobs

cat > infra/images/lora_training/dockerfile <<'DOCKER'
FROM python:3.11-slim

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1
ENV PIP_NO_CACHE_DIR=1

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/ai_platform

COPY lora/training/requirements.txt /tmp/requirements.txt

RUN python -m pip install --upgrade pip setuptools wheel && \
    python -m pip install -r /tmp/requirements.txt

CMD ["python", "--version"]
DOCKER
```

## 2. Create image build script

```bash
# on laptop
cd ~/workdir/ai_platform

cat > scripts/build_and_push_lora_training_image.sh <<'EOF_SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

REGISTRY=192.168.178.103:5000
IMAGE=$REGISTRY/lora-trainer:latest

docker buildx build \
  --platform linux/arm64 \
  -f infra/images/lora_training/dockerfile \
  -t "$IMAGE" \
  --push .
EOF_SCRIPT

chmod +x scripts/build_and_push_lora_training_image.sh
```

## 3. Create CPU Slurm training job

The training script uses environment variables, so the Slurm job exports the same variables used during manual training.

```bash
# on laptop
cd ~/workdir/ai_platform

cat > infra/slurm/jobs/lora-train.slurm <<'EOF_SLURM'
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
EOF_SLURM
```

## 4. Build and push CPU training image

```bash
# on laptop
cd ~/workdir/ai_platform

./scripts/build_and_push_lora_training_image.sh
```

## 5. Pull CPU image into Apptainer

```bash
# on jetson
mkdir -p /home/roman/nfs/lora/images

apptainer pull \
  --no-https \
  /home/roman/nfs/lora/images/lora-trainer.sif \
  docker://192.168.178.103:5000/lora-trainer:latest

ls -lh /home/roman/nfs/lora/images/lora-trainer.sif
```

## 6. Submit CPU LoRA training job

The job file is copied manually into NFS, following the same style as the RAG Slurm job.

```bash
# on raspberry
mkdir -p /home/roman/nfs/lora/jobs
mkdir -p /home/roman/nfs/lora/logs
mkdir -p /home/roman/nfs/lora/adapters

sudo vi /home/roman/nfs/lora/jobs/lora-train.slurm

ls -lh /home/roman/nfs/lora/jobs/lora-train.slurm
ls -lh /home/roman/nfs/lora/images/lora-trainer.sif

sbatch /home/roman/nfs/lora/jobs/lora-train.slurm

squeue

tail -f /home/roman/nfs/lora/logs/lora-train-*.out
tail -f /home/roman/nfs/lora/logs/lora-train-*.err
```

Expected adapter output:

```text
/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>
```

## 7. Convert Slurm-trained adapter to GGUF

```bash
# on jetson
cd ~/workdir/llama.cpp
source ~/workdir/llama.cpp/.venv-lora/bin/activate

python convert_lora_to_gguf.py \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID> \
  --base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct \
  --outfile /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>.gguf \
  --outtype f16

ls -lh /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>*
```

Confirmed result:

```text
Model successfully exported to:
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_37.gguf
```

## 8. GPU investigation

GPU inference is already active in K3s.

The llama.cpp deployment uses:

```yaml
runtimeClassName: nvidia
```

and:

```text
-ngl 99
```

So:

```text
K3s llama.cpp chat inference: GPU
K3s llama.cpp embeddings: GPU
RAG rebuild: CPU Slurm job calling GPU embedding server
LoRA CPU training: CPU Slurm + Apptainer
LoRA GPU training: experimental
```

## 9. Inspect Jetson CUDA stack

```bash
# on jetson
cat /etc/nv_tegra_release

dpkg -l | grep -E "nvidia-l4t-core|cuda|cudnn|tensorrt" | head -50

nvcc --version || true

nvidia-smi || true

python3 - <<'PY'
import platform
print("machine:", platform.machine())
PY
```

Observed stack:

```text
Jetson L4T R36.4.7
CUDA 12.6
aarch64
NVIDIA-SMI works on host
```

## 10. Verify Slurm GPU resource

```bash
# on raspberry
sinfo -o "%N %G %t"

scontrol show node jetson | grep -E "Gres|CfgTRES|AllocTRES"
```

Expected:

```text
jetson gpu:1 idle
Gres=gpu:1
```

## 11. GPU smoke job

A simple Slurm GPU smoke job confirmed that Slurm schedules GPU jobs onto Jetson and that GPU device files are visible.

```bash
# on raspberry
sudo vi /home/roman/nfs/lora/jobs/gpu-smoke.slurm

sbatch /home/roman/nfs/lora/jobs/gpu-smoke.slurm

squeue

tail -f /home/roman/nfs/lora/logs/gpu-smoke-*.out
```

Observed device files:

```text
/dev/nvidia0
/dev/nvidiactl
/dev/nvidia-modeset
/dev/nvhost-gpu
/dev/nvhost-ctrl-gpu
/dev/nvhost-as-gpu
/dev/nvhost-ctxsw-gpu
```

## 12. Create CUDA LoRA image

The first NVIDIA NGC tag tried did not exist:

```text
nvcr.io/nvidia/l4t-pytorch:r36.4.0-pth2.5-py3
```

The working base image used for the experiment was:

```text
dustynv/l4t-pytorch:r36.4.0
```

```bash
# on laptop
cd ~/workdir/ai_platform

mkdir -p infra/images/lora_training_cuda

cat > infra/images/lora_training_cuda/dockerfile <<'DOCKER'
FROM dustynv/l4t-pytorch:r36.4.0

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1
ENV PIP_NO_CACHE_DIR=1
ENV PIP_INDEX_URL=https://pypi.org/simple
ENV PIP_EXTRA_INDEX_URL=https://pypi.ngc.nvidia.com

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/ai_platform

RUN python3 -m pip install --upgrade pip setuptools wheel

RUN python3 -m pip install \
    transformers \
    peft \
    datasets \
    accelerate \
    safetensors \
    sentencepiece \
    scipy

CMD ["python3", "-c", "import torch; print(torch.__version__); print(torch.cuda.is_available())"]
DOCKER
```

```bash
# on laptop
cd ~/workdir/ai_platform

cat > scripts/build_and_push_lora_training_cuda_image.sh <<'EOF_SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

REGISTRY=192.168.178.103:5000
IMAGE=$REGISTRY/lora-trainer-cuda:latest

docker buildx build \
  --platform linux/arm64 \
  -f infra/images/lora_training_cuda/dockerfile \
  -t "$IMAGE" \
  --push .
EOF_SCRIPT

chmod +x scripts/build_and_push_lora_training_cuda_image.sh

./scripts/build_and_push_lora_training_cuda_image.sh
```

The CUDA image is much larger than the CPU image.

Observed rough size:

```text
CPU image:  about 1.6 GB
CUDA image: about 13 GB
```

This is expected because the CUDA image includes Jetson CUDA/PyTorch runtime libraries.

## 13. Pull CUDA image into Apptainer

```bash
# on jetson
apptainer pull \
  --no-https \
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif \
  docker://192.168.178.103:5000/lora-trainer-cuda:latest

ls -lh /home/roman/nfs/lora/images/lora-trainer-cuda.sif
```

## 14. Manual CUDA visibility test with Apptainer

This command works from a normal Jetson shell:

```bash
# on jetson
apptainer exec \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /dev/nvidia0:/dev/nvidia0 \
  --bind /dev/nvidiactl:/dev/nvidiactl \
  --bind /dev/nvidia-modeset:/dev/nvidia-modeset \
  --bind /dev/nvhost-ctrl-gpu:/dev/nvhost-ctrl-gpu \
  --bind /dev/nvhost-as-gpu:/dev/nvhost-as-gpu \
  --bind /dev/nvhost-gpu:/dev/nvhost-gpu \
  --bind /dev/nvhost-ctxsw-gpu:/dev/nvhost-ctxsw-gpu \
  --bind /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu:ro \
  --bind /usr/local/cuda:/usr/local/cuda:ro \
  --env LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu/nvidia:/usr/local/cuda/lib64 \
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif \
  python3 -c "import torch; print(torch.__version__); print(torch.version.cuda); print(torch.cuda.is_available()); print(torch.cuda.device_count())"
```

Observed:

```text
2.4.0
12.6
True
1
```

A Docker runtime test also worked:

```bash
# on jetson
docker run --rm --runtime nvidia \
  192.168.178.103:5000/lora-trainer-cuda:latest \
  python3 -c "import torch; print(torch.__version__); print(torch.version.cuda); print(torch.cuda.is_available()); print(torch.cuda.device_count())"
```

Observed:

```text
2.4.0
12.6
True
1
```

## 15. Slurm GPU Apptainer attempt

A GPU Slurm job was created to run the CUDA SIF on Jetson:

```bash
# on laptop
cd ~/workdir/ai_platform

cat > infra/slurm/jobs/lora-train-gpu.slurm <<'EOF_SLURM'
#!/usr/bin/env bash
#SBATCH --job-name=lora-train-gpu
#SBATCH --output=/home/roman/nfs/lora/logs/lora-train-gpu-%j.out
#SBATCH --error=/home/roman/nfs/lora/logs/lora-train-gpu-%j.err
#SBATCH --nodes=1
#SBATCH --nodelist=jetson
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=6G
#SBATCH --gres=gpu:1
#SBATCH --time=02:00:00

set -euo pipefail

SIF="/home/roman/nfs/lora/images/lora-trainer-cuda.sif"
TRAIN_SCRIPT="/home/roman/workdir/ai_platform/lora/training/train_qwen_lora_gpu.py"

export BASE_MODEL="/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct"
export DATASET_PATH="/home/roman/nfs/lora/datasets/lab_style.jsonl"
export OUTPUT_DIR="/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_${SLURM_JOB_ID}"
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
echo "SLURM CUDA_VISIBLE_DEVICES=${CUDA_VISIBLE_DEVICES:-unset}"
echo "OUTPUT_DIR=${OUTPUT_DIR}"

unset CUDA_VISIBLE_DEVICES

echo "Container CUDA check"

apptainer exec \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  --bind /dev/nvidia0:/dev/nvidia0 \
  --bind /dev/nvidiactl:/dev/nvidiactl \
  --bind /dev/nvidia-modeset:/dev/nvidia-modeset \
  --bind /dev/nvhost-ctrl-gpu:/dev/nvhost-ctrl-gpu \
  --bind /dev/nvhost-as-gpu:/dev/nvhost-as-gpu \
  --bind /dev/nvhost-gpu:/dev/nvhost-gpu \
  --bind /dev/nvhost-ctxsw-gpu:/dev/nvhost-ctxsw-gpu \
  --bind /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu:ro \
  --bind /usr/local/cuda:/usr/local/cuda:ro \
  --env LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu/nvidia:/usr/local/cuda/lib64 \
  "${SIF}" \
  python3 -c "import torch, transformers; print(torch.__version__); print(transformers.__version__); print(torch.version.cuda); print(torch.cuda.is_available()); print(torch.cuda.device_count())"

echo "Starting GPU LoRA training"

apptainer exec \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  --bind /dev/nvidia0:/dev/nvidia0 \
  --bind /dev/nvidiactl:/dev/nvidiactl \
  --bind /dev/nvidia-modeset:/dev/nvidia-modeset \
  --bind /dev/nvhost-ctrl-gpu:/dev/nvhost-ctrl-gpu \
  --bind /dev/nvhost-as-gpu:/dev/nvhost-as-gpu \
  --bind /dev/nvhost-gpu:/dev/nvhost-gpu \
  --bind /dev/nvhost-ctxsw-gpu:/dev/nvhost-ctxsw-gpu \
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
  "${SIF}" \
  python3 "${TRAIN_SCRIPT}"

echo "GPU adapter written to ${OUTPUT_DIR}"
EOF_SLURM
```

```bash
# on raspberry
sudo vi ~/nfs/lora/jobs/lora-train-gpu.slurm

sbatch /home/roman/nfs/lora/jobs/lora-train-gpu.slurm

squeue

tail -f /home/roman/nfs/lora/logs/lora-train-gpu-*.out
tail -f /home/roman/nfs/lora/logs/lora-train-gpu-*.err
```

Observed issue:

```text
Inside Slurm-launched Apptainer:
  torch: 2.4.0
  CUDA build: 12.6
  cuda available: False
  device count: 0
```

An interactive `srun --pty bash` on Jetson also failed with:

```text
CUDA error 801: operation not supported
Can't initialize NVML
```

This means the issue is not the image alone. It is related to the Slurm-launched process/session on Jetson.

## 16. SSH workaround test

Running the same Apptainer command through SSH from Raspberry to Jetson worked:

```bash
# on raspberry
ssh \
  -i /home/roman/.ssh/raspberry_to_jetson \
  -o StrictHostKeyChecking=accept-new \
  roman@jetson \
  'bash -lc '"'"'
    unset CUDA_VISIBLE_DEVICES
    unset NVIDIA_VISIBLE_DEVICES

    apptainer exec \
      --bind /home/roman/nfs:/home/roman/nfs \
      --bind /dev/nvidia0:/dev/nvidia0 \
      --bind /dev/nvidiactl:/dev/nvidiactl \
      --bind /dev/nvidia-modeset:/dev/nvidia-modeset \
      --bind /dev/nvhost-ctrl-gpu:/dev/nvhost-ctrl-gpu \
      --bind /dev/nvhost-as-gpu:/dev/nvhost-as-gpu \
      --bind /dev/nvhost-gpu:/dev/nvhost-gpu \
      --bind /dev/nvhost-ctxsw-gpu:/dev/nvhost-ctxsw-gpu \
      --bind /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu:ro \
      --bind /usr/local/cuda:/usr/local/cuda:ro \
      --env LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu/nvidia:/usr/local/cuda/lib64 \
      /home/roman/nfs/lora/images/lora-trainer-cuda.sif \
      python3 -c "import torch; print(torch.__version__); print(torch.version.cuda); print(torch.cuda.is_available()); print(torch.cuda.device_count())"
  '"'"''
```

Observed:

```text
2.4.0
12.6
True
1
```

This confirms:

```text
Raspberry -> SSH -> Jetson -> Apptainer CUDA: works
Jetson login shell -> Apptainer CUDA: works
Jetson Slurm step -> Apptainer CUDA: fails
```

## 17. GPU training script experiment

A GPU-specific training script was prepared:

```bash
# on laptop
cd ~/workdir/ai_platform

cat > lora/training/train_qwen_lora_gpu.py <<'PY'
import json
import os
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, get_peft_model
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
)

BASE_MODEL = os.environ.get(
    "BASE_MODEL",
    "/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct",
)
DATASET_PATH = os.environ.get(
    "DATASET_PATH",
    "/home/roman/nfs/lora/datasets/lab_style.jsonl",
)
OUTPUT_DIR = os.environ.get(
    "OUTPUT_DIR",
    "/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu",
)
MAX_LENGTH = int(os.environ.get("MAX_LENGTH", "256"))


def load_jsonl(path: str):
    rows = []

    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()

            if not line:
                continue

            rows.append(json.loads(line))

    return rows


def main():
    print(f"BASE_MODEL={BASE_MODEL}")
    print(f"DATASET_PATH={DATASET_PATH}")
    print(f"OUTPUT_DIR={OUTPUT_DIR}")
    print(f"CUDA={torch.cuda.is_available()}")
    print(f"CUDA_DEVICE_COUNT={torch.cuda.device_count()}")

    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is not available inside the training container")

    Path(OUTPUT_DIR).mkdir(parents=True, exist_ok=True)

    tokenizer = AutoTokenizer.from_pretrained(BASE_MODEL, trust_remote_code=True)

    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    rows = load_jsonl(DATASET_PATH)
    dataset = Dataset.from_list(rows)

    def tokenize(example):
        text = tokenizer.apply_chat_template(
            example["messages"],
            tokenize=False,
            add_generation_prompt=False,
        )

        encoded = tokenizer(
            text,
            truncation=True,
            max_length=MAX_LENGTH,
            padding="max_length",
        )

        encoded["labels"] = encoded["input_ids"].copy()
        return encoded

    tokenized = dataset.map(tokenize, remove_columns=dataset.column_names)

    model = AutoModelForCausalLM.from_pretrained(
        BASE_MODEL,
        trust_remote_code=True,
        torch_dtype=torch.float16,
        device_map={"": 0},
    )

    model.config.use_cache = False

    lora_config = LoraConfig(
        r=8,
        lora_alpha=16,
        target_modules=[
            "q_proj",
            "k_proj",
            "v_proj",
            "o_proj",
            "gate_proj",
            "up_proj",
            "down_proj",
        ],
        lora_dropout=0.05,
        bias="none",
        task_type="CAUSAL_LM",
    )

    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    args = TrainingArguments(
        output_dir=OUTPUT_DIR,
        num_train_epochs=float(os.environ.get("NUM_TRAIN_EPOCHS", "1")),
        max_steps=int(os.environ.get("MAX_STEPS", "1")),
        per_device_train_batch_size=int(os.environ.get("BATCH_SIZE", "1")),
        gradient_accumulation_steps=int(os.environ.get("GRAD_ACCUM_STEPS", "1")),
        learning_rate=float(os.environ.get("LEARNING_RATE", "2e-4")),
        logging_steps=1,
        save_steps=1,
        save_total_limit=1,
        report_to=[],
        fp16=True,
    )

    trainer = Trainer(
        model=model,
        args=args,
        train_dataset=tokenized,
    )

    trainer.train()

    model.save_pretrained(OUTPUT_DIR)
    tokenizer.save_pretrained(OUTPUT_DIR)

    print(f"Saved GPU LoRA adapter to {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
PY
```

Manual GPU training command:

```bash
# on jetson
export BASE_MODEL="/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct"
export DATASET_PATH="/home/roman/nfs/lora/datasets/lab_style.jsonl"
export OUTPUT_DIR="/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_manual"
export MAX_STEPS="50"
export NUM_TRAIN_EPOCHS="20"
export MAX_LENGTH="256"
export LEARNING_RATE="2e-4"
export BATCH_SIZE="1"
export GRAD_ACCUM_STEPS="1"

mkdir -p /home/roman/nfs/lora/adapters

apptainer exec \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  --bind /dev/nvidia0:/dev/nvidia0 \
  --bind /dev/nvidiactl:/dev/nvidiactl \
  --bind /dev/nvidia-modeset:/dev/nvidia-modeset \
  --bind /dev/nvhost-ctrl-gpu:/dev/nvhost-ctrl-gpu \
  --bind /dev/nvhost-as-gpu:/dev/nvhost-as-gpu \
  --bind /dev/nvhost-gpu:/dev/nvhost-gpu \
  --bind /dev/nvhost-ctxsw-gpu:/dev/nvhost-ctxsw-gpu \
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
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif \
  python3 /home/roman/workdir/ai_platform/lora/training/train_qwen_lora_gpu.py
```

This currently fails because the CUDA image installed latest `transformers`:

```text
transformers 5.8.0
torch 2.4.0
```

Error:

```text
ValueError: infer_schema(func): Parameter input has unsupported type torch.Tensor
```

The next fix is to rebuild the CUDA image with pinned package versions compatible with Jetson PyTorch 2.4.0.

Planned package pinning:

```text
transformers==4.44.2
peft==0.12.0
datasets==2.21.0
accelerate==0.33.0
```

## Current conclusion

Working:

```text
CPU LoRA training:
  Slurm -> Apptainer -> train_qwen_lora.py -> PEFT adapter

LoRA conversion:
  PEFT adapter -> llama.cpp convert_lora_to_gguf.py -> GGUF LoRA

GPU inference:
  K3s -> llama.cpp -> NVIDIA runtime -> Jetson GPU

Manual CUDA visibility:
  Jetson shell -> Apptainer CUDA image -> torch cuda True
```

Not completed yet:

```text
GPU LoRA training:
  Needs CUDA image package pinning

GPU LoRA through Slurm:
  Slurm-launched Jetson session causes CUDA error 801 / cuda False
```

Next session:

```text
1. Rebuild lora-trainer-cuda image with pinned Python packages.
2. Pull new lora-trainer-cuda.sif on Jetson.
3. Retry manual GPU LoRA training through Apptainer.
4. Convert GPU-trained adapter to GGUF.
5. Continue investigating direct Slurm GPU Apptainer execution.
6. Add K3s llama-cpp-lora serving manifest.
```
