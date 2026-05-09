# 🧠 AI Platform Lab

![Rust](https://img.shields.io/badge/Rust-Systems%20Programming-black?logo=rust)
![K3s](https://img.shields.io/badge/K3s-Kubernetes-blue)
![Slurm](https://img.shields.io/badge/Slurm-HPC-green)
![llama.cpp](https://img.shields.io/badge/llama.cpp-GGUF-orange)
![RAG](https://img.shields.io/badge/RAG-Retrieval-purple)
![LoRA](https://img.shields.io/badge/LoRA-Fine--Tuning-red)
![Jetson](https://img.shields.io/badge/NVIDIA-Jetson-76B900?logo=nvidia)
![ARM64](https://img.shields.io/badge/ARM64-Edge%20Compute-lightgrey)

Practical AI infrastructure and orchestration lab built with Rust, Slurm, K3s, llama.cpp, RAG, and LoRA workflows on ARM64 hardware.

This repository combines:

* Rust-based AI orchestration
* RAG pipelines
* MCP and HTTP tools
* llama.cpp inference
* Slurm batch scheduling
* K3s orchestration
* NFS shared storage
* LoRA fine-tuning workflows
* GPU-backed inference on edge hardware

The goal is not to build a polished SaaS product.

The goal is to understand how modern AI infrastructure layers fit together in practice using real heterogeneous hardware.

---

# 🎯 Project Goals

This repository serves as a hands-on engineering lab for:

* AI infrastructure
* distributed systems
* GPU inference
* retrieval systems
* orchestration
* batch scheduling
* model specialization
* OpenAI-compatible serving

The platform is intentionally built incrementally and kept observable.

Main design goals:

* swappable model backends
* explicit infrastructure layers
* reproducible workflows
* minimal hidden magic
* practical deployment experience

---

# 🖥 Hardware Layout

| Device | Role |
|---|---|
| Laptop | development + builds + control |
| Raspberry Pi 4 | Slurm controller + K3s control-plane |
| Jetson Orin Nano | GPU worker + inference + NFS server |

---

# 🌐 Runtime Endpoints

Main external API:

```text
http://100.109.72.92:30080
```

Internal inference services:

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
/home/roman/nfs
```

---

# 🧭 High-Level Architecture

```text
                Client
                   │
                   ▼

         ┌──────────────────┐
         │ ai-platform-host │
         │ Rust orchestrator│
         └────────┬─────────┘
                  │
      ┌───────────┼────────────┐
      ▼           ▼            ▼

┌──────────┐ ┌──────────┐ ┌────────────┐
│ RAG      │ │ Tools    │ │ llama.cpp │
│ Retrieval│ │ MCP/HTTP │ │ runtimes  │
└────┬─────┘ └────┬─────┘ └─────┬──────┘
     │             │             │
     └─────────────┼─────────────┘
                   ▼

          ┌────────────────┐
          │ Shared NFS     │
          │ /home/roman/nfs│
          └──────┬─────────┘
                 ▼

          ┌────────────────┐
          │ Slurm + GPU    │
          │ batch jobs     │
          └────────────────┘
```

---

# 🔀 Current Runtime Layout

| Service | Purpose |
|---|---|
| llama-cpp | base inference |
| llama-cpp-lora | LoRA behavior specialization |
| llama-cpp-embed | embeddings |
| ai-platform-host | orchestration layer |

---

# 🧠 Online vs Offline Split

## 🟢 Online Serving Plane

```text
Client
  ↓
Rust host
  ↓
retrieval / tools / routing
  ↓
llama.cpp runtime
  ↓
response
```

Responsibilities:

* HTTP API
* orchestration
* retrieval
* tool execution
* backend selection
* response routing

---

## 🔵 Offline Batch Plane

```text
Dataset / documents
  ↓
Slurm job
  ↓
Apptainer container
  ↓
artifacts / adapters
  ↓
NFS shared storage
```

Responsibilities:

* LoRA training
* RAG indexing
* artifact generation
* future automation jobs

---

# 📦 Workspace Structure

```text
workspace/
├── host/              # Rust orchestration layer
├── tools_server/      # MCP / HTTP tools
├── llm_client/        # LLM abstraction layer
├── shared_types/      # shared models
├── indexer/           # RAG indexing
├── knowledge_base/    # markdown KB
├── artifacts/         # generated RAG artifacts
├── lora/              # LoRA datasets + training
└── infra/             # Slurm + K3s + NFS
```

---

# 🧩 Current Infrastructure State

# Slurm

* Raspberry Pi → controller
* Jetson → worker
* CPU + GPU scheduling
* Apptainer integration

---

# NFS

* Jetson → NFS server
* Raspberry + K3s → clients

Shared root:

```text
/home/roman/nfs
```

Used for:

* models
* datasets
* LoRA adapters
* RAG artifacts
* logs
* Apptainer images

---

# K3s

| Node | Role |
|---|---|
| Raspberry Pi | control-plane |
| Jetson | GPU worker |

Current workloads:

* ai-platform-host
* llama.cpp runtimes
* embedding services
* warmup jobs

---

# 📦 Storage Layout

```text
/ home / roman / nfs/
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

# 🧠 RAG Architecture

Current RAG implementation intentionally avoids external vector databases.

Artifacts are stored as JSON:

```text
artifacts/rag/
├── chunks.json
├── embeddings.json
└── manifest.json
```

Current retrieval flow:

```text
User query
  ↓
embeddings
  ↓
similarity search
  ↓
context injection
  ↓
generation
```

Design goals:

* debuggable
* observable
* easy to rebuild
* minimal dependencies

---

# 🔥 LoRA Workflow

Current LoRA pipeline supports:

* CPU training
* GPU training
* Slurm scheduling
* Apptainer execution
* GGUF conversion
* llama.cpp runtime loading

Flow:

```text
Dataset
  ↓
Trainer
  ↓
LoRA adapter
  ↓
GGUF conversion
  ↓
llama.cpp runtime
```

Current interpretation of LoRA:

* style specialization
* formatting behavior
* future tool-call discipline

NOT:

* primary factual memory
* replacement for retrieval

RAG still provides factual grounding.

---

# 🔀 Runtime Routing

The Rust orchestration layer supports multiple runtime profiles.

Current routing:

| Profile | Runtime |
|---|---|
| default | llama-cpp |
| lora | llama-cpp-lora |

Current response modes:

| Mode | Behavior |
|---|---|
| agent | retrieval + tools |
| direct | direct generation |

Example:

```json
{
  "message": "What is Slurm?",
  "llm_profile": "lora",
  "response_mode": "direct"
}
```

---

# 🧪 Current Working Features

Implemented and working:

* GPU inference ✔
* GPU LoRA training ✔
* Slurm scheduling ✔
* Apptainer GPU execution ✔
* OpenAI-compatible APIs ✔
* Rust orchestration ✔
* K3s deployment ✔
* RAG retrieval ✔
* embedding generation ✔
* LoRA runtime routing ✔
* direct response mode ✔
* MCP tools ✔

---

# ⚠️ Operational Notes

Jetson Orin Nano has limited VRAM/RAM.

During GPU LoRA training:

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
* keep systems observable
* prefer explicit infrastructure
* avoid unnecessary abstraction
* use standard APIs
* build incrementally
* optimize later

---

# 🗺 Roadmap

Possible future directions:

* automated Slurm workflows
* structured tool-call tuning
* monitoring/metrics
* vector database integration
* distributed training experiments
* lightweight UI
* multi-model routing

---

# 📌 Summary

This repository now demonstrates a practical miniature AI platform with:

* distributed compute
* shared storage
* K3s orchestration
* Slurm scheduling
* GPU inference
* GPU fine-tuning
* RAG pipelines
* LoRA specialization
* Rust orchestration
* OpenAI-compatible serving

built incrementally on real ARM hardware infrastructure.
