# 🖥 Infra Track: Slurm + NFS + K3s (AI Platform Lab)

This directory tracks the infrastructure side of the AI platform project.

The goal is to understand how:

* distributed compute
* GPU inference
* batch scheduling
* shared storage
* orchestration
* retrieval pipelines
* LoRA specialization

fit together inside a practical heterogeneous ARM lab.

The project intentionally focuses on:
- observability
- incremental architecture
- explicit infrastructure layers
- OpenAI-compatible interfaces
- reproducible local workflows

---

# 🎯 Current Platform State

The platform currently demonstrates:

* distributed ARM infrastructure
* GPU-backed llama.cpp inference
* Slurm batch scheduling
* Apptainer execution
* K3s orchestration
* RAG indexing pipelines
* embedding generation
* LoRA fine-tuning
* LoRA GGUF conversion
* multi-runtime inference routing
* Rust orchestration layer

using:

* Laptop
* Raspberry Pi 4
* Jetson Orin Nano

---

# 🌐 External Access

Main external API endpoint:

```text
http://100.109.72.92:30080
```

Internal K3s services:

```text
http://llama-cpp:8000
http://llama-cpp-lora:8003
http://llama-cpp-embed:8001
```

Docker registry:

```text
192.168.178.103:5000
```

Shared NFS root:

```text
/ home / roman / nfs
```

---

# 🧭 High-Level Architecture

```text
                ┌────────────────────┐
                │ ai-platform-host   │
                │ Rust orchestration │
                └─────────┬──────────┘
                          │
      ┌───────────────────┼────────────────────┐
      ▼                   ▼                    ▼

┌──────────────┐  ┌────────────────┐  ┌────────────────┐
│ llama-cpp    │  │ llama-cpp-lora │  │ llama-cpp-embed│
│ port 8000    │  │ port 8003      │  │ port 8001      │
│ base model   │  │ LoRA runtime   │  │ embeddings     │
└──────┬───────┘  └────────┬───────┘  └────────┬───────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           ▼

                 ┌──────────────────┐
                 │ NFS Shared Store │
                 │ /home/roman/nfs  │
                 └────────┬─────────┘
                          │
                          ▼

                 ┌──────────────────┐
                 │ Slurm + GPU Jobs │
                 │ Jetson Worker    │
                 └──────────────────┘
```

---

# 🧩 Infrastructure Layers

# 1️⃣ Serving Plane

The Rust orchestration layer handles:

* HTTP API
* request routing
* retrieval orchestration
* MCP/tool execution
* backend selection
* direct vs agent response modes

Runs inside K3s.

---

# 2️⃣ Inference Plane

llama.cpp runtimes provide:

* OpenAI-compatible APIs
* GGUF inference
* GPU acceleration
* embeddings
* LoRA adapter loading

Current runtimes:

| Runtime | Purpose |
|---|---|
| llama-cpp | base inference |
| llama-cpp-lora | LoRA behavior specialization |
| llama-cpp-embed | embeddings |

---

# 3️⃣ Training Plane

Slurm + Apptainer handle:

* LoRA fine-tuning
* dataset processing
* RAG indexing
* artifact generation
* future automation pipelines

Training can run:

* CPU-only
* GPU-accelerated (`apptainer exec --nv`)

---

# 4️⃣ Storage Plane

Shared NFS storage:

```text
/home/roman/nfs
```

Used for:

* GGUF models
* HuggingFace models
* LoRA adapters
* datasets
* embeddings
* RAG artifacts
* Slurm logs
* Apptainer images

---

# 5️⃣ Orchestration Plane

K3s cluster:

| Node | Role |
|---|---|
| Raspberry Pi | control-plane |
| Jetson Orin Nano | GPU worker |

K3s runs:

* Rust host
* llama.cpp runtimes
* embedding services
* warmup jobs
* future services

---

# 📦 Storage Layout

```text
/home/roman/nfs/
├── models/
│   ├── gguf/
│   └── huggingface/
├── rag/
│   ├── knowledge_base/
│   ├── artifacts/
│   └── images/
├── lora/
│   ├── datasets/
│   ├── adapters/
│   ├── images/
│   └── jobs/
└── logs/
```

---

# 🔀 Runtime Routing

The Rust host supports multiple inference profiles.

Current routing:

| Profile | Backend |
|---|---|
| default | llama-cpp |
| lora | llama-cpp-lora |

Current response modes:

| Mode | Behavior |
|---|---|
| agent | retrieval + tools |
| direct | direct backend generation |

Example:

```json
{
  "message": "What is Slurm?",
  "llm_profile": "lora",
  "response_mode": "direct"
}
```

---

# 🧠 LoRA Interpretation

LoRA is currently used primarily for:

* formatting specialization
* answer style tuning
* behavioral shaping
* future JSON/tool discipline

LoRA is NOT treated as:

* authoritative factual memory
* replacement for retrieval
* replacement for tools

Factual grounding should still come from:

* RAG
* tools
* retrieval context

---

# 🔥 End-to-End Flows

# Online Inference Flow

```text
Client
  ↓
ai-platform-host
  ↓
retrieval / routing
  ↓
llama.cpp runtime
  ↓
response
```

---

# Offline Training Flow

```text
Dataset
  ↓
Slurm job
  ↓
Apptainer container
  ↓
LoRA adapter
  ↓
GGUF conversion
  ↓
K3s deployment
```

---

# 🧪 Current Infrastructure Capabilities

Implemented and working:

* GPU inference ✔
* GPU LoRA training ✔
* Slurm scheduling ✔
* Apptainer GPU execution ✔
* llama.cpp serving ✔
* embedding runtime ✔
* RAG indexing ✔
* OpenAI-compatible APIs ✔
* Rust orchestration ✔
* K3s deployment ✔
* LoRA routing ✔
* direct response mode ✔

---

# ⚠️ Operational Notes

# GPU Resource Limits

Jetson Orin Nano has limited VRAM/RAM.

Before GPU training:

```bash
kubectl scale deployment llama-cpp -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-lora -n ai-platform --replicas=0
kubectl scale deployment llama-cpp-embed -n ai-platform --replicas=0
```

After training:

```bash
kubectl scale deployment llama-cpp -n ai-platform --replicas=1
kubectl scale deployment llama-cpp-lora -n ai-platform --replicas=1
kubectl scale deployment llama-cpp-embed -n ai-platform --replicas=1
```

---

# 🧭 Design Principles

* separate online/offline workloads
* prefer explicit infrastructure
* avoid unnecessary abstraction
* use standard APIs
* keep systems observable
* prefer reproducible workflows
* build incrementally

---

# 🗺 Roadmap

Possible future directions:

* automated Slurm pipelines
* structured tool-call tuning
* vector database integration
* metrics/monitoring
* distributed training experiments
* lightweight UI
* multi-model routing

---

# 📌 Summary

This repository now demonstrates a practical miniature AI infrastructure platform with:

* heterogeneous ARM hardware
* GPU inference
* GPU fine-tuning
* Slurm orchestration
* K3s orchestration
* shared storage
* RAG
* embeddings
* Rust orchestration
* LoRA specialization
* OpenAI-compatible APIs

built incrementally on real hardware infrastructure.
