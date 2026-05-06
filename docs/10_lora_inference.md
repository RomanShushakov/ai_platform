# 🧪 LoRA Inference Proof with llama.cpp

This step proves the full LoRA adapter path end-to-end:

```text
dataset
  ↓
PEFT LoRA training
  ↓
adapter_model.safetensors
  ↓
convert_lora_to_gguf.py
  ↓
GGUF LoRA adapter
  ↓
llama.cpp --lora
  ↓
changed model behavior
```

This is not yet the final Slurm-based training pipeline.  
The goal of this step is to prove that a locally trained adapter can be loaded by `llama.cpp`.

---

# 🎯 Goal

After this step, you will have:

- LoRA support confirmed in your llama.cpp build
- versioned LoRA dataset in the repo
- LoRA assets synced to NFS
- a tiny PEFT adapter trained from the dataset
- a GGUF LoRA adapter created
- llama.cpp serving the base model with the adapter
- visible behavior change in responses

---

# 🧭 Architecture

```text
Laptop
  ↓
repo/lora/datasets/lab_style.jsonl
  ↓ sync script
NFS
  ↓
Jetson
  ↓ train_qwen_lora.py
PEFT adapter
  ↓
convert_lora_to_gguf.py
  ↓
GGUF LoRA adapter
  ↓
llama.cpp --lora
  ↓
Raspberry curl test
```

---

# 1️⃣ Check llama.cpp LoRA Support

### on jetson:

```bash
~/workdir/llama.cpp/build/bin/llama-server --help | grep -i lora -A8 -B4

~/workdir/llama.cpp/build/bin/llama-export-lora --help | head -80 || true
~/workdir/llama.cpp/build/bin/llama-finetune --help | head -80 || true
```

Expected flags:

```text
--lora FNAME
--lora-scaled FNAME:SCALE
--lora-init-without-apply
```

---

# 2️⃣ Create NFS LoRA Directories

### on jetson:

```bash
mkdir -p /home/roman/nfs/models/lora
mkdir -p /home/roman/nfs/lora/datasets
mkdir -p /home/roman/nfs/lora/adapters
mkdir -p /home/roman/nfs/lora/jobs/logs
mkdir -p /home/roman/nfs/lora/jobs/work
```

---

# 3️⃣ Inspect llama.cpp LoRA Tools

### on jetson:

```bash
ls -lh ~/workdir/llama.cpp/build/bin | grep -Ei 'lora|finetune|convert|export'
```

Check converter scripts:

```bash
find ~/workdir/llama.cpp -maxdepth 3 -type f | grep -Ei 'lora|convert'
```

Inspect llama.cpp LoRA conversion test:

```bash
sed -n '1,220p' ~/workdir/llama.cpp/tests/test-lora-conversion-inference.sh
```

Inspect converter requirements:

```bash
cat ~/workdir/llama.cpp/requirements/requirements-convert_lora_to_gguf.txt
```

---

# 4️⃣ Create LoRA Conversion Virtual Environment

This environment is for **conversion**, not training.

### on jetson:

```bash
python3 -m venv ~/workdir/llama.cpp/.venv-lora

source ~/workdir/llama.cpp/.venv-lora/bin/activate

pip install -r ~/workdir/llama.cpp/requirements/requirements-convert_lora_to_gguf.txt
```

---

# 5️⃣ Optional: Search for Existing Compatible Adapters

### on jetson, inside `.venv-lora`:

```bash
python - <<'PY'
from huggingface_hub import list_models

models = list_models(
    search="Qwen2.5-0.5B-Instruct LoRA",
    limit=10
)

for m in models:
    print(m.modelId)
PY
```

The project continued with a self-created adapter because it is more reproducible and controlled.

---

# 6️⃣ Prepare Repo Dataset

The dataset is stored in the repo first, then synced to NFS.

### on laptop:

```bash
mkdir -p lora/datasets

cat > lora/datasets/lab_style.jsonl <<'EOF'
{"messages":[{"role":"user","content":"What is Kubernetes?"},{"role":"assistant","content":"Lab note: Kubernetes is a system for running and managing containerized applications across machines."}]}
{"messages":[{"role":"user","content":"What is Slurm?"},{"role":"assistant","content":"Lab note: Slurm is a workload manager used to schedule batch jobs on compute nodes."}]}
{"messages":[{"role":"user","content":"What is NFS?"},{"role":"assistant","content":"Lab note: NFS is shared network storage that lets multiple machines read and write the same files."}]}
{"messages":[{"role":"user","content":"What is RAG?"},{"role":"assistant","content":"Lab note: RAG combines retrieval from documents with LLM generation to answer using external context."}]}
{"messages":[{"role":"user","content":"What is llama.cpp?"},{"role":"assistant","content":"Lab note: llama.cpp is a lightweight runtime for serving GGUF language models efficiently on local hardware."}]}
EOF
```

This dataset intentionally trains a simple style:

```text
Lab note: ...
```

It is not intended to inject factual knowledge. Facts should still come from RAG.

---

# 7️⃣ Add Sync Script for LoRA Assets

### on laptop:

```bash
cat > scripts/sync_lora_assets_to_nfs.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

REMOTE_USER=roman
REMOTE_HOST=raspberry
REMOTE_DIR=/home/roman/nfs/lora
SSH_PRIVATE_KEY_FILE=/home/roman/.ssh/raspberry

echo "Creating LoRA directories on NFS..."

ssh -i ${SSH_PRIVATE_KEY_FILE} ${REMOTE_USER}@${REMOTE_HOST} "
  mkdir -p '${REMOTE_DIR}/datasets'
  mkdir -p '${REMOTE_DIR}/adapters'
  mkdir -p '${REMOTE_DIR}/scripts'
  mkdir -p '${REMOTE_DIR}/jobs/logs'
  mkdir -p '${REMOTE_DIR}/jobs/work'
"

echo "Syncing LoRA datasets..."

rsync -avz \
  lora/datasets/ \
  "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/datasets/"

echo "Done."
EOF

chmod +x scripts/sync_lora_assets_to_nfs.sh
```

Run sync:

```bash
./scripts/sync_lora_assets_to_nfs.sh
```

Verify:

### on raspberry:

```bash
ls -lh /home/roman/nfs/lora/datasets
cat /home/roman/nfs/lora/datasets/lab_style.jsonl
```

---

# 8️⃣ Create LoRA Training Requirements

This first training environment is a **CPU smoke-test environment**. GPU training will be moved into a proper container/Slurm job later.

### on laptop:

```bash
mkdir -p lora/training
mkdir -p lora/jobs

cat > lora/training/requirements.txt <<'EOF'
torch==2.6.0
transformers
peft
datasets
accelerate
safetensors
sentencepiece
scipy
EOF
```

Sync the updated file to Jetson using the normal repo sync workflow.

---

# 9️⃣ Create Training Virtual Environment

### on jetson:

```bash
cd ~/workdir/ai_platform

python3 -m venv lora/training/.venv

source lora/training/.venv/bin/activate

pip install --upgrade pip

pip install -r lora/training/requirements.txt
```

Verify:

```bash
python - <<'PY'
import torch
import transformers
import peft
import datasets

print("torch:", torch.__version__)
print("cuda:", torch.cuda.is_available())
print("transformers:", transformers.__version__)
print("peft ok")
print("datasets ok")
PY
```

Expected for this CPU smoke-test venv:

```text
torch: 2.6.0+cpu
cuda: False
```

---

# 🔟 Create Training Script

### on laptop:

```bash
cat > lora/training/train_qwen_lora.py <<'PY'
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

BASE_MODEL = os.environ.get("BASE_MODEL", "Qwen/Qwen2.5-0.5B-Instruct")
DATASET_PATH = os.environ.get(
    "DATASET_PATH",
    "/home/roman/nfs/lora/datasets/lab_style.jsonl",
)
OUTPUT_DIR = os.environ.get(
    "OUTPUT_DIR",
    "/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b",
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
        dtype=torch.float32,
        device_map=None,
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
        use_cpu=True,
    )

    trainer = Trainer(
        model=model,
        args=args,
        train_dataset=tokenized,
    )

    trainer.train()

    model.save_pretrained(OUTPUT_DIR)
    tokenizer.save_pretrained(OUTPUT_DIR)

    print(f"Saved LoRA adapter to {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
PY
```

Sync the updated file to Jetson using the normal repo sync workflow.

---

# 1️⃣1️⃣ Run 1-Step Smoke Training

### on jetson:

```bash
cd ~/workdir/ai_platform
source lora/training/.venv/bin/activate

MAX_STEPS=1 \
NUM_TRAIN_EPOCHS=1 \
DATASET_PATH=/home/roman/nfs/lora/datasets/lab_style.jsonl \
OUTPUT_DIR=/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_smoke \
python lora/training/train_qwen_lora.py
```

Verify adapter output:

```bash
ls -lh /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_smoke
```

Expected files include:

```text
adapter_config.json
adapter_model.safetensors
tokenizer_config.json
```

This 1-step adapter validates the pipeline, but the behavior change may be weak.

---

# 1️⃣2️⃣ Download Local Hugging Face Base Model

The LoRA converter needs a local HF base model directory.

### on jetson:

```bash
mkdir -p /home/roman/nfs/models/huggingface

deactivate || true

source ~/workdir/llama.cpp/.venv-lora/bin/activate
```

Download base model:

```bash
hf download Qwen/Qwen2.5-0.5B-Instruct \
  --local-dir /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct
```

---

# 1️⃣3️⃣ Convert 1-Step Adapter to GGUF

### on jetson:

```bash
cd ~/workdir/llama.cpp

python convert_lora_to_gguf.py \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_smoke \
  --base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct \
  --outfile /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_smoke.gguf \
  --outtype f16
```

Verify:

```bash
ls -lh /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_smoke.gguf
```

---

# 1️⃣4️⃣ Test 1-Step Adapter in llama.cpp

### on jetson:

```bash
docker run --rm -it \
  --runtime nvidia \
  --network host \
  -v /home/roman/nfs/models/gguf:/models \
  -v /home/roman/nfs/lora/adapters:/adapters \
  192.168.178.103:5000/llama-cpp-server:latest \
  -m /models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --lora /adapters/lab_style_qwen2_5_0_5b_smoke.gguf \
  --host 0.0.0.0 \
  --port 8002 \
  -ngl 99 \
  -c 1024
```

Test:

### on raspberry:

```bash
curl -X POST http://jetson:8002/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages":[
      {
        "role":"user",
        "content":"What is Slurm?"
      }
    ],
    "temperature":0,
    "max_tokens":64
  }'
```

Expected: the adapter loads, but style change may be weak because training used only one step.

---

# 1️⃣5️⃣ Train Stronger 50-Step Adapter

### on jetson:

```bash
deactivate || true

cd ~/workdir/ai_platform
source lora/training/.venv/bin/activate

MAX_STEPS=50 \
NUM_TRAIN_EPOCHS=20 \
DATASET_PATH=/home/roman/nfs/lora/datasets/lab_style.jsonl \
OUTPUT_DIR=/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_steps50 \
python lora/training/train_qwen_lora.py
```

---

# 1️⃣6️⃣ Convert 50-Step Adapter to GGUF

### on jetson:

```bash
deactivate || true

cd ~/workdir/llama.cpp
source ~/workdir/llama.cpp/.venv-lora/bin/activate

python convert_lora_to_gguf.py \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_steps50 \
  --base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct \
  --outfile /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_steps50.gguf \
  --outtype f16
```

---

# 1️⃣7️⃣ Run 50-Step LoRA Adapter

### on jetson:

```bash
docker run --rm -it \
  --runtime nvidia \
  --network host \
  -v /home/roman/nfs/models/gguf:/models \
  -v /home/roman/nfs/lora/adapters:/adapters \
  192.168.178.103:5000/llama-cpp-server:latest \
  -m /models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --lora /adapters/lab_style_qwen2_5_0_5b_steps50.gguf \
  --host 0.0.0.0 \
  --port 8003 \
  -ngl 99 \
  -c 1024
```

---

# 1️⃣8️⃣ Verify LoRA Behavior

### on raspberry:

Test Slurm:

```bash
curl -X POST http://jetson:8003/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages":[
      {"role":"user","content":"What is Slurm?"}
    ],
    "temperature":0,
    "max_tokens":48
  }'
```

Test Kubernetes:

```bash
curl -X POST http://jetson:8003/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages":[
      {"role":"user","content":"What is Kubernetes?"}
    ],
    "temperature":0,
    "max_tokens":48
  }'
```

Test RAG:

```bash
curl -X POST http://jetson:8003/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages":[
      {"role":"user","content":"What is RAG?"}
    ],
    "temperature":0,
    "max_tokens":48
  }'
```

Test nearby unseen prompt:

```bash
curl -X POST http://jetson:8003/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages":[
      {"role":"user","content":"Explain NFS briefly."}
    ],
    "temperature":0,
    "max_tokens":48
  }'
```

Expected style:

```text
Lab note: ...
```

---

# 🧪 Observed Results

The 1-step smoke adapter successfully loaded but did not strongly change behavior.

The 50-step adapter produced the desired style:

```text
Lab note: ...
```

This proves that the LoRA adapter is actually influencing model behavior.

---

# 🔥 Troubleshooting

## `TrainingArguments.__init__()` rejects `overwrite_output_dir`

Remove:

```python
overwrite_output_dir=True
```

The used `transformers` version did not support it.

---

## `TrainingArguments.__init__()` rejects `no_cuda`

Use:

```python
use_cpu=True
```

instead of:

```python
no_cuda=True
```

---

## Converter fails with `FileNotFoundError: Qwen/Qwen2.5-0.5B-Instruct`

The converter needs a local base model directory.

Fix:

```bash
hf download Qwen/Qwen2.5-0.5B-Instruct \
  --local-dir /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct
```

Then pass:

```bash
--base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct
```

---

## 1-step adapter does not visibly affect output

Expected. Use more steps:

```bash
MAX_STEPS=50
NUM_TRAIN_EPOCHS=20
```

---

# 📌 Summary

You now have:

- LoRA support confirmed in llama.cpp ✔
- repo-managed dataset ✔
- LoRA assets synced to NFS ✔
- PEFT adapter trained ✔
- adapter converted to GGUF ✔
- llama.cpp loaded adapter ✔
- behavior changed successfully ✔

---

# 🚀 Next Step

👉 Promote LoRA training into a Slurm/Apptainer pipeline.
