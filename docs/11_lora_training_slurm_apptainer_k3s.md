<!--
AI Platform Lab Documentation
Standardized Edition
Environment:
- Laptop: development machine
- Raspberry Pi: Slurm controller + K3s control-plane
- Jetson Orin Nano: GPU worker + inference + training node
- External Tailscale endpoint: 100.109.72.92
-->

# 🧬 LoRA Training with Slurm, Apptainer, and K3s Serving

This guide promotes the LoRA proof from manual/local training into a cluster-style workflow.

It covers:

* CPU LoRA training through Slurm + Apptainer
* CUDA LoRA training through Slurm + Apptainer on Jetson
* adapter conversion to GGUF
* serving the trained LoRA adapter through llama.cpp in K3s
* operational lessons for sharing one Jetson GPU between inference and training

---

# 🎯 Goal

By the end of this guide, you will have:

* reproducible CPU LoRA training through Slurm
* reproducible GPU LoRA training through Slurm
* training images stored in the local registry
* Apptainer SIF images stored on NFS
* adapters written to NFS
* GGUF LoRA adapters generated with llama.cpp
* a `llama-cpp-lora` K3s service serving the trained adapter
* a clear workflow for switching between inference and training on the Jetson GPU

---

# 🧭 Architecture

```text
Laptop
  ↓ buildx push
Local registry
  ↓ apptainer pull
NFS image store
  ↓
Slurm job on Raspberry
  ↓ schedules on Jetson
Apptainer training container
  ↓
PEFT adapter on NFS
  ↓ convert_lora_to_gguf.py
GGUF LoRA adapter
  ↓
K3s llama-cpp-lora service
```

---

# 📁 Important Paths

```text
Repo:
  ~/workdir/ai_platform

NFS root:
  /home/roman/nfs

LoRA dataset:
  /home/roman/nfs/lora/datasets/lab_style.jsonl

HF base model:
  /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct

GGUF base model:
  /home/roman/nfs/models/gguf/qwen2.5-0.5b-instruct-q4_k_m.gguf

CPU training SIF:
  /home/roman/nfs/lora/images/lora-trainer.sif

CUDA training SIF:
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif

Adapter output:
  /home/roman/nfs/lora/adapters

Slurm LoRA jobs:
  /home/roman/nfs/lora/jobs

LoRA logs:
  /home/roman/nfs/lora/logs

llama.cpp checkout:
  ~/workdir/llama.cpp
```

---

# 1️⃣ Repository Files

The real files live in the repo and are synced to the machines that execute them.

## ▶️ Files Added or Updated

```text
infra/images/lora_training/dockerfile
infra/images/lora_training_cuda/dockerfile

infra/slurm/jobs/lora-train.slurm
infra/slurm/jobs/lora-train-gpu.slurm

lora/training/train_qwen_lora.py
lora/training/train_qwen_lora_gpu.py
lora/training/requirements.txt

scripts/build_and_push_lora_training_image.sh
scripts/build_and_push_lora_training_cuda_image.sh

scripts/sync_lora_training_to_jetson.sh
scripts/sync_lora_slurm_jobs_to_nfs.sh
scripts/sync_rag_slurm_jobs_to_nfs.sh

infra/k3s/manifests/ai_platform/llama-cpp-lora.yml
```

---

# 2️⃣ Build CPU LoRA Training Image

The CPU image is used for a safe Slurm smoke/stability path.

## ▶️ On Laptop

```bash
cd ~/workdir/ai_platform

./scripts/build_and_push_lora_training_image.sh
```

Expected image:

```text
192.168.178.103:5000/lora-trainer:latest
```

---

# 3️⃣ Pull CPU Image into Apptainer

## ▶️ On Jetson

```bash
mkdir -p /home/roman/nfs/lora/images

apptainer pull \
  --no-https \
  /home/roman/nfs/lora/images/lora-trainer.sif \
  docker://192.168.178.103:5000/lora-trainer:latest

ls -lh /home/roman/nfs/lora/images/lora-trainer.sif
```

---

# 4️⃣ Sync CPU/GPU Training Files

The Slurm jobs call training scripts from:

```text
/home/roman/workdir/ai_platform/lora/training
```

So training scripts are synced from laptop to Jetson.

## ▶️ On Laptop

```bash
cd ~/workdir/ai_platform

chmod +x scripts/sync_lora_training_to_jetson.sh

./scripts/sync_lora_training_to_jetson.sh
```

Verify:

## ▶️ On Jetson

```bash
ls -lh /home/roman/workdir/ai_platform/lora/training/
```

---

# 5️⃣ Sync LoRA Slurm Jobs to NFS

LoRA Slurm jobs are stored in the repo first, then synced to NFS for execution from Raspberry.

## ▶️ On Laptop

```bash
cd ~/workdir/ai_platform

chmod +x scripts/sync_lora_slurm_jobs_to_nfs.sh

./scripts/sync_lora_slurm_jobs_to_nfs.sh
```

Verify:

## ▶️ On Raspberry

```bash
ls -lh /home/roman/nfs/lora/jobs/
```

Expected:

```text
lora-train.slurm
lora-train-gpu.slurm
```

---

# 6️⃣ Run CPU LoRA Training through Slurm

CPU training is useful as a stable baseline and smoke path.

The stable CPU profile uses:

```text
cpus-per-task: 2
memory: 4G
MAX_STEPS=20
NUM_TRAIN_EPOCHS=10
MAX_LENGTH=64
OMP_NUM_THREADS=2
MKL_NUM_THREADS=2
```

This avoids overloading the Jetson while still proving the full training path.

## ▶️ On Raspberry

```bash
sbatch /home/roman/nfs/lora/jobs/lora-train.slurm

squeue

tail -f /home/roman/nfs/lora/logs/lora-train-*.out
tail -f /home/roman/nfs/lora/logs/lora-train-*.err
```

Expected output includes:

```text
CUDA=False
train_runtime
Saved LoRA adapter to /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>
Adapter written to /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>
```

Confirmed stable CPU adapter:

```text
/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_91
/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_91.gguf
```

---

# 7️⃣ Convert CPU Adapter to GGUF

## ▶️ On Jetson

```bash
cd ~/workdir/llama.cpp

source ~/workdir/llama.cpp/.venv-lora/bin/activate

python convert_lora_to_gguf.py \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID> \
  --base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct \
  --outfile /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>.gguf \
  --outtype f16

ls -lh /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_slurm_<JOB_ID>*
```

---

# 8️⃣ Build CUDA LoRA Training Image

GPU training uses a Jetson-specific CUDA/PyTorch image.

The working base image is:

```text
dustynv/l4t-pytorch:r36.4.0
```

The Python packages are pinned because newer `transformers` versions were incompatible with the Jetson PyTorch 2.4.0 image.

Pinned versions:

```text
transformers==4.44.2
peft==0.12.0
datasets==2.21.0
accelerate==0.33.0
```

## ▶️ On Laptop

```bash
cd ~/workdir/ai_platform

./scripts/build_and_push_lora_training_cuda_image.sh
```

Expected image:

```text
192.168.178.103:5000/lora-trainer-cuda:latest
```

---

# 9️⃣ Pull CUDA Image into Apptainer

## ▶️ On Jetson

```bash
rm -f /home/roman/nfs/lora/images/lora-trainer-cuda.sif

apptainer pull \
  --no-https \
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif \
  docker://192.168.178.103:5000/lora-trainer-cuda:latest

ls -lh /home/roman/nfs/lora/images/lora-trainer-cuda.sif
```

The CUDA image is much larger than the CPU image.

Observed rough size:

```text
CPU image:  about 1.6 GB
CUDA image: about 13 GB
```

This is expected because the CUDA image includes Jetson CUDA/PyTorch runtime components.

---

# 🔟 Verify CUDA Image

## ▶️ On Jetson

Use `sudo apptainer` because GPU access through Apptainer on Jetson requires root in this setup.

```bash
sudo apptainer exec \
  --nv \
  --bind /home/roman/nfs:/home/roman/nfs \
  --bind /home/roman/workdir:/home/roman/workdir \
  --bind /usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu:ro \
  --bind /usr/local/cuda:/usr/local/cuda:ro \
  --env LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu:/usr/lib/aarch64-linux-gnu/nvidia:/usr/local/cuda/lib64 \
  /home/roman/nfs/lora/images/lora-trainer-cuda.sif \
  python3 -c "import torch, transformers; print(torch.__version__); print(transformers.__version__); print(torch.version.cuda); print(torch.cuda.is_available()); print(torch.cuda.device_count())"
```

Expected:

```text
2.4.0
4.44.2
12.6
True
1
```

---

# 1️⃣1️⃣ Allow Slurm Job to Use sudo Apptainer

GPU training uses:

```text
sudo -n apptainer exec --nv
```

so `roman` needs passwordless sudo for Apptainer.

## ▶️ On Jetson

```bash
sudo visudo -f /etc/sudoers.d/roman-apptainer
```

Add:

```text
roman ALL=(root) NOPASSWD: /usr/bin/apptainer
```

Verify:

```bash
sudo -n apptainer --version
```

---

# 1️⃣2️⃣ Jetson GPU Device Notes

On this Jetson, `/dev/nvidia0` is not always present after reboot.

The stable Jetson GPU device stack is usually:

```text
/dev/nvhost-*
/dev/nvgpu/...
/dev/nvmap
```

At one point, `/dev/nvidia0` existed and was used by explicit Apptainer bind mounts. Later, after reboot, it disappeared.

The final working approach is:

```text
sudo apptainer exec --nv
```

instead of manually binding `/dev/nvidia0`.

---

# 1️⃣3️⃣ Restore Jetson GPU Permissions After Reboot

After reboot, non-root GPU access can fail with:

```text
NvRmMemInitNvmap failed with Permission denied
```

Temporary fix:

## ▶️ On Jetson

```bash
sudo chgrp -R video /dev/nvgpu
sudo chmod -R g+rwX /dev/nvgpu

sudo chgrp video /dev/nvmap /dev/nvhost-*gpu*
sudo chmod 660 /dev/nvmap /dev/nvhost-*gpu*

nvidia-smi
```

Expected:

```text
nvidia-smi works without sudo
```

If only `sudo nvidia-smi` works, check permissions again:

```bash
ls -l /dev/nvmap /dev/nvhost-*gpu*
sudo find /dev/nvgpu -ls
```

---

# 1️⃣4️⃣ Slurm GRES Configuration

GRES lets Slurm schedule GPU jobs onto Jetson.

Verify:

## ▶️ On Raspberry

```bash
sinfo -o "%N %G %t"

scontrol show node jetson | grep -E "Gres|CfgTRES|AllocTRES"
```

Expected:

```text
jetson gpu:1 idle
Gres=gpu:1
```

If Jetson was rebooted and Slurm marks it down:

```bash
sudo /opt/slurm/bin/scontrol update NodeName=jetson State=RESUME

/opt/slurm/bin/sinfo -N -l
```

---

# 1️⃣5️⃣ Important GPU Scheduling Rule

The Jetson GPU is shared by:

```text
K3s llama.cpp inference
Slurm LoRA training
```

If the llama.cpp pods are running, GPU LoRA training can fail with:

```text
CUDA error: out of memory
```

So before GPU LoRA training, scale inference pods down.

## ▶️ On Raspberry

```bash
kubectl scale deployment llama-cpp -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-lora -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-embed -n ai-platform --replicas=0

kubectl get pods -n ai-platform -o wide
```

After training, scale them back up:

```bash
kubectl scale deployment llama-cpp -n ai-platform --replicas=1
kubectl scale deployment llama-cpp-lora -n ai-platform --replicas=1
kubectl scale deployment llama-cpp-embed -n ai-platform --replicas=1

kubectl get pods -n ai-platform -o wide
```

This is the final operational lesson:

```text
Inference and training share one Jetson GPU.
Do not run both at the same time on this small node.
```

---

# 1️⃣6️⃣ Run GPU LoRA Training through Slurm

The stable GPU Slurm profile uses:

```text
cpus-per-task: 2
memory: 4G
MAX_STEPS=20
NUM_TRAIN_EPOCHS=10
MAX_LENGTH=64
BATCH_SIZE=1
PYTORCH_NO_CUDA_MEMORY_CACHING=1
CUDA_MODULE_LOADING=LAZY
sudo apptainer exec --nv
```

## ▶️ On Laptop

Sync the final job:

```bash
cd ~/workdir/ai_platform

./scripts/sync_lora_slurm_jobs_to_nfs.sh
```

## ▶️ On Raspberry

Scale inference down first:

```bash
kubectl scale deployment llama-cpp -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-lora -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-embed -n ai-platform --replicas=0
```

Submit GPU training:

```bash
sbatch /home/roman/nfs/lora/jobs/lora-train-gpu.slurm

squeue

tail -f /home/roman/nfs/lora/logs/lora-train-gpu-*.out
tail -f /home/roman/nfs/lora/logs/lora-train-gpu-*.err
```

Expected output:

```text
Container CUDA check
2.4.0
4.44.2
12.6
True
1

Starting GPU LoRA training
CUDA=True
CUDA_DEVICE_COUNT=1
trainable params: 4,399,104 || all params: 498,431,872 || trainable%: 0.8826
Saved GPU LoRA adapter to /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID>
GPU adapter written to /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID>
```

Confirmed stable GPU adapter:

```text
/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_97
```

---

# 1️⃣7️⃣ Convert GPU Adapter to GGUF

## ▶️ On Jetson

```bash
cd ~/workdir/llama.cpp

source ~/workdir/llama.cpp/.venv-lora/bin/activate

python convert_lora_to_gguf.py \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID> \
  --base /home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct \
  --outfile /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID>.gguf \
  --outtype f16

ls -lh /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID>*
```

Fix ownership because training uses `sudo apptainer`:

```bash
sudo chown -R roman:roman \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID>

sudo chown roman:roman \
  /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_<JOB_ID>.gguf
```

Confirmed stable GGUF adapter:

```text
/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_97.gguf
```

---

# 1️⃣8️⃣ Serve GPU-Trained Adapter Manually

Manual serving is useful before K3s deployment.

## ▶️ On Jetson

```bash
cd ~/workdir/llama.cpp

./build/bin/llama-server \
  -m /home/roman/nfs/models/gguf/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --lora /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_97.gguf \
  --host 0.0.0.0 \
  --port 8004 \
  -ngl 99 \
  -c 1024
```

## ▶️ On Laptop

```bash
curl http://100.109.72.92:8004/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-lora-gpu",
    "messages": [
      {
        "role": "user",
        "content": "What is Slurm?"
      }
    ],
    "temperature": 0.2
  }'
```

Expected style:

```text
Lab note: ...
```

---

# 1️⃣9️⃣ Deploy `llama-cpp-lora` to K3s

The LoRA serving manifest is separate from the base `llama-cpp` manifest.

This keeps:

```text
llama-cpp        base chat endpoint
llama-cpp-lora   LoRA chat endpoint
llama-cpp-embed  embeddings endpoint
```

## ▶️ On Laptop

```bash
cd ~/workdir/ai_platform

./scripts/sync_k3s_manifests.sh
```

## ▶️ On Raspberry

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/llama-cpp-lora.yml

kubectl get pods -n ai-platform -o wide

kubectl get svc -n ai-platform

kubectl logs -n ai-platform deploy/llama-cpp-lora
```

Expected service:

```text
llama-cpp-lora   ClusterIP   ...   8003/TCP
```

---

# 2️⃣0️⃣ Test `llama-cpp-lora` Inside K3s

## ▶️ On Raspberry

```bash
kubectl run curl-lora-test \
  -n ai-platform \
  --rm -it \
  --image=curlimages/curl:latest \
  --restart=Never \
  -- \
  curl -s http://llama-cpp-lora:8003/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{
      "model": "qwen-lora-gpu",
      "messages": [
        {
          "role": "user",
          "content": "What is Slurm?"
        }
      ],
      "temperature": 0.2
    }'
```

Observed example:

```text
Lab note: Slurm is a job scheduler for Linux systems.
```

Other prompts may still fall back to base-model style because this is a tiny dataset and short training run.

This is acceptable for the platform milestone.

The important proof is infrastructural:

```text
Slurm GPU training
  ↓
GGUF LoRA adapter
  ↓
K3s llama.cpp LoRA serving
  ↓
OpenAI-compatible endpoint
```

---

# 2️⃣1️⃣ Adapter Cleanup

Keep stable artifacts:

```text
lab_style_qwen2_5_0_5b_steps50/
lab_style_qwen2_5_0_5b_steps50.gguf

lab_style_qwen2_5_0_5b_slurm_91/
lab_style_qwen2_5_0_5b_slurm_91.gguf

lab_style_qwen2_5_0_5b_gpu_slurm_97/
lab_style_qwen2_5_0_5b_gpu_slurm_97.gguf
```

Remove failed partial attempts:

## ▶️ On Jetson

```bash
sudo rm -rf /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_93
sudo rm -rf /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_94
sudo rm -rf /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_95
sudo rm -rf /home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu_slurm_96
```

Verify:

```bash
ls -lh /home/roman/nfs/lora/adapters/
```

---

# 🔥 Troubleshooting

## CUDA works manually but fails in Slurm

Use:

```text
sudo -n apptainer exec --nv
```

not manual `/dev/nvidia0` binds.

The final working Slurm GPU path uses:

```text
sudo apptainer
Apptainer --nv
K3s inference pods scaled to zero
```

---

## `/dev/nvidia0` disappeared

On Jetson, `/dev/nvidia0` is not the stable source of truth.

Use:

```text
/dev/nvhost-*
/dev/nvgpu/...
/dev/nvmap
```

and rely on:

```text
apptainer --nv
```

for GPU injection.

---

## `NvRmMemInitNvmap failed with Permission denied`

Fix device permissions:

```bash
sudo chgrp -R video /dev/nvgpu
sudo chmod -R g+rwX /dev/nvgpu

sudo chgrp video /dev/nvmap /dev/nvhost-*gpu*
sudo chmod 660 /dev/nvmap /dev/nvhost-*gpu*
```

---

## GPU training hits CUDA OOM

Scale K3s inference down first:

```bash
kubectl scale deployment llama-cpp -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-lora -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-embed -n ai-platform --replicas=0
```

Use conservative training settings:

```text
MAX_LENGTH=64
MAX_STEPS=20
BATCH_SIZE=1
PYTORCH_NO_CUDA_MEMORY_CACHING=1
```

---

## CUDA image fails with `transformers::grouped_mm_fallback`

Use pinned Python packages:

```text
transformers==4.44.2
peft==0.12.0
datasets==2.21.0
accelerate==0.33.0
```

---

## Adapter files are owned by root

Because GPU training uses `sudo apptainer`.

Fix:

```bash
sudo chown -R roman:roman /home/roman/nfs/lora/adapters/<ADAPTER_DIR>
sudo chown roman:roman /home/roman/nfs/lora/adapters/<ADAPTER_FILE>.gguf
```

---

# 📌 Summary

You now have:

* CPU LoRA training through Slurm + Apptainer ✔
* GPU LoRA training through Slurm + Apptainer ✔
* pinned CUDA training image ✔
* adapters written to NFS ✔
* GGUF LoRA conversion ✔
* K3s `llama-cpp-lora` serving ✔
* operational workflow for scaling inference down during training ✔

Final confirmed flow:

```text
Laptop builds image
  ↓
Registry
  ↓
Jetson pulls SIF
  ↓
Raspberry submits Slurm job
  ↓
Jetson trains adapter
  ↓
NFS stores adapter
  ↓
llama.cpp converts to GGUF
  ↓
K3s serves LoRA endpoint
```

---

# 🚀 Next Step

👉 Extend `ai-platform-host` so it can route between:

```text
LLM_BASE_URL=http://llama-cpp:8000
LLM_LORA_BASE_URL=http://llama-cpp-lora:8003
EMBEDDING_BASE_URL=http://llama-cpp-embed:8001
```

Planned routing:

```text
normal chat / RAG:
  llama-cpp

lab-style or formatting-sensitive mode:
  llama-cpp-lora

embeddings:
  llama-cpp-embed
```
