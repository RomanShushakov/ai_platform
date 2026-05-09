<!--
AI Platform Lab Documentation
Standardized Edition
Environment:
- Laptop: development machine
- Raspberry Pi: Slurm controller + K3s control-plane
- Jetson Orin Nano: GPU worker + inference + training node
- External Tailscale endpoint: 100.109.72.92
-->

# 🧪 LoRA Inference Proof with llama.cpp

This step proves the full LoRA adapter path end-to-end:

```text
Dataset
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

This chapter intentionally keeps training simple and reproducible.

The goal is:

* verify LoRA support in llama.cpp
* verify PEFT training works
* verify GGUF conversion works
* verify llama.cpp can load LoRA adapters
* verify behavior changes

GPU Slurm/Apptainer training is moved into the next chapter.

---

# 🎯 Goal

After this step, you will have:

* LoRA support confirmed in llama.cpp
* repo-managed LoRA datasets
* LoRA assets synchronized to NFS
* a CPU-trained PEFT adapter
* a GGUF LoRA adapter
* llama.cpp serving the adapter
* visible model behavior change

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

# 1️⃣ Verify llama.cpp LoRA Support

## ▶️ On Jetson

Verify LoRA flags exist:

```bash
~/workdir/llama.cpp/build/bin/llama-server --help | grep -i lora -A8 -B4
```

Expected flags:

```text
--lora FNAME
--lora-scaled FNAME:SCALE
--lora-init-without-apply
```

Inspect optional tooling:

```bash
~/workdir/llama.cpp/build/bin/llama-export-lora --help | head -80 || true
~/workdir/llama.cpp/build/bin/llama-finetune --help | head -80 || true
```

Inspect conversion helpers:

```bash
ls -lh ~/workdir/llama.cpp/build/bin | grep -Ei 'lora|finetune|convert|export'

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

# 2️⃣ Prepare NFS LoRA Directories

## ▶️ On Jetson

```bash
mkdir -p /home/roman/nfs/models/lora
mkdir -p /home/roman/nfs/models/huggingface
mkdir -p /home/roman/nfs/lora/datasets
mkdir -p /home/roman/nfs/lora/adapters
mkdir -p /home/roman/nfs/lora/jobs/logs
mkdir -p /home/roman/nfs/lora/jobs/work
```

---

# 3️⃣ Prepare Repository Structure

## ▶️ Files Added

```text
lora/datasets/lab_style.jsonl
lora/training/requirements.txt
lora/training/train_qwen_lora.py
scripts/sync_lora_assets_to_nfs.sh
```

The dataset intentionally trains a style pattern:

```text
Lab note: ...
```

The adapter is not intended to inject factual knowledge.

RAG remains responsible for knowledge retrieval.

---

# 4️⃣ Create LoRA Conversion Environment

This environment is only used for:

```text
convert_lora_to_gguf.py
```

## ▶️ On Jetson

```bash
python3 -m venv ~/workdir/llama.cpp/.venv-lora

source ~/workdir/llama.cpp/.venv-lora/bin/activate

pip install -r ~/workdir/llama.cpp/requirements/requirements-convert_lora_to_gguf.txt
```

---

# 5️⃣ Synchronize LoRA Assets to NFS

## ▶️ On Laptop

The sync workflow uses:

```text
scripts/sync_lora_assets_to_nfs.sh
```

Run:

```bash
./scripts/sync_lora_assets_to_nfs.sh
```

Verify:

## ▶️ On Raspberry

```bash
ls -lh /home/roman/nfs/lora/datasets

cat /home/roman/nfs/lora/datasets/lab_style.jsonl
```

---

# 6️⃣ Create CPU Training Environment

This environment is intentionally CPU-only.

GPU Slurm/Apptainer training is implemented later.

## ▶️ On Jetson

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

print('torch:', torch.__version__)
print('cuda:', torch.cuda.is_available())
print('transformers:', transformers.__version__)
print('peft ok')
print('datasets ok')
PY
```

Expected:

```text
torch: 2.6.0+cpu
cuda: False
```

---

# 7️⃣ Run Smoke-Test Training

## ▶️ On Jetson

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

Expected files:

```text
adapter_config.json
adapter_model.safetensors
tokenizer_config.json
```

---

# 8️⃣ Download Local Hugging Face Base Model

The converter requires a local HF model directory.

## ▶️ On Jetson

```bash
source ~/workdir/llama.cpp/.venv-lora/bin/activate
```

Download:

```bash
hf download Qwen/Qwen2.5-0.5B-Instruct \
  --local-dir /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct
```

---

# 9️⃣ Convert Smoke Adapter to GGUF

## ▶️ On Jetson

```bash
cd ~/workdir/llama.cpp

source ~/workdir/llama.cpp/.venv-lora/bin/activate

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

# 🔟 Run Smoke Adapter in llama.cpp

## ▶️ On Jetson

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

## ▶️ On Raspberry

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

The smoke adapter validates:

```text
PEFT → GGUF → llama.cpp
```

The behavior change may still be weak.

---

# 1️⃣1️⃣ Train Stronger Adapter

## ▶️ On Jetson

```bash
cd ~/workdir/ai_platform

source lora/training/.venv/bin/activate

MAX_STEPS=50 \
NUM_TRAIN_EPOCHS=20 \
DATASET_PATH=/home/roman/nfs/lora/datasets/lab_style.jsonl \
OUTPUT_DIR=/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_steps50 \
python lora/training/train_qwen_lora.py
```

---

# 1️⃣2️⃣ Convert Stronger Adapter to GGUF

## ▶️ On Jetson

```bash
cd ~/workdir/llama.cpp

source ~/workdir/llama.cpp/.venv-lora/bin/activate

python convert_lora_to_gguf.py \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_steps50 \
  --base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct \
  --outfile /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_steps50.gguf \
  --outtype f16
```

---

# 1️⃣3️⃣ Run Stronger Adapter in llama.cpp

## ▶️ On Jetson

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

# 1️⃣4️⃣ Verify LoRA Behavior

## ▶️ On Raspberry

```bash
curl -X POST http://jetson:8003/v1/chat/completions \
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
    "max_tokens":48
  }'
```

Expected style:

```text
Lab note: ...
```

Example:

```text
Lab note: Slurm is a workload manager used to schedule batch jobs on compute nodes.
```

---

# 🧪 Observed Results

The smoke adapter successfully loaded but produced only weak behavioral changes.

The stronger adapter produced the intended style:

```text
Lab note: ...
```

This confirmed:

```text
PEFT adapter
  ↓
GGUF conversion
  ↓
llama.cpp --lora
  ↓
behavior modification
```

works correctly.

---

# 🔥 Troubleshooting

## `overwrite_output_dir` rejected

Older transformers versions rejected:

```python
overwrite_output_dir=True
```

The field was removed.

---

## `no_cuda=True` rejected

Use:

```python
use_cpu=True
```

instead of:

```python
no_cuda=True
```

---

## Converter cannot find base model

Error:

```text
FileNotFoundError
```

Fix:

```bash
hf download Qwen/Qwen2.5-0.5B-Instruct \
  --local-dir /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct
```

Use:

```bash
--base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct
```

---

## Smoke adapter barely changes behavior

Expected.

Increase:

```bash
MAX_STEPS=50
NUM_TRAIN_EPOCHS=20
```

---

# 📌 Summary

You now have:

* LoRA support confirmed in llama.cpp ✔
* repo-managed datasets ✔
* synchronized LoRA assets ✔
* PEFT adapter training ✔
* GGUF conversion ✔
* llama.cpp LoRA loading ✔
* observable behavior change ✔

---

# 🚀 Next Step

👉 Move LoRA training into:

```text
Slurm + Apptainer + GPU
```

with:

* containerized training
* reproducible environments
* GPU scheduling
* NFS-based adapter storage
* production-style workflows
