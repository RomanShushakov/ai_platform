# 🖥 Infra Track: Slurm + NFS + K3s (AI Platform Lab)

This directory tracks the infrastructure side of the AI platform project.

The goal is not just to run a chatbot, but to understand how:

* online serving
* batch compute
* shared storage
* container orchestration
* GPU inference backends

fit together in a real, minimal lab environment.

---

# 🎯 Architecture Overview

```text
           ┌───────────────┐
           │   Laptop      │
           │ curl / dev    │
           └──────┬────────┘
                  │
                  ▼
        ┌─────────────────────┐
        │      K3s Cluster    │
        │ (Raspberry + Jetson)│
        └────────┬────────────┘
                 │
     ┌───────────┼──────────────┐
     ▼                           ▼
┌──────────────┐         ┌──────────────┐
│ Rust Host    │         │ llama.cpp    │
│ (API + MCP)  │         │ (LLM Server) │
└──────┬───────┘         └──────┬───────┘
       │                        │
       ▼                        ▼
        ┌─────────────────────┐
        │     NFS Storage     │
        │ (Jetson /home/nfs)  │
        └────────┬────────────┘
                 │
                 ▼
           ┌───────────────┐
           │    Slurm      │
           │ Batch Jobs    │
           └───────────────┘
```

---

# 🧩 Architecture Layers

## 1. Serving Plane (Rust AI Platform)

Handles:

* chat API
* tool orchestration (MCP / HTTP)
* retrieval (RAG)
* LLM interaction (OpenAI-compatible)

Runs inside K3s.

Supports multiple backends:

* llama.cpp (current primary)
* Ollama (fallback / legacy)

---

## 2. LLM Serving Plane (llama.cpp)

Handles:

* model inference (GGUF)
* OpenAI-compatible API (`/v1/chat/completions`)
* embeddings (planned)
* LoRA support (future)

Key characteristics:

* runs directly on Jetson GPU (CUDA)
* deployed as K3s workload
* uses local GGUF models from NFS

---

## 3. Batch Plane (Slurm)

Handles:

* RAG artifact generation
* embedding jobs
* preprocessing
* future training / fine-tuning

Runs on Jetson (GPU-capable node).

---

## 4. Storage Plane (NFS)

Shared directory:

```
/home/roman/nfs
```

Used for:

* GGUF models
* RAG artifacts
* embeddings
* job outputs
* logs

Mounted inside K3s pods:

```
/models
/mnt/nfs
```

---

## 5. Orchestration Plane (K3s)

Cluster:

* Raspberry → control-plane
* Jetson → GPU worker

Runs:

* Rust host
* llama.cpp server
* Ollama (optional / legacy)
* warmup jobs
* future services

---

# 📦 Storage Layout

```
/home/roman/nfs/
├── slurm/
│   └── job-examples/
├── rag/
│   ├── knowledge_base/
│   └── artifacts/
│       ├── chunks.json
│       ├── embeddings.json
│       └── manifest.json
├── models/
│   └── gguf/
│       └── *.gguf
└── logs/
```

---

# 🔥 End-to-End Data Flow

## Online Flow (Current)

```text
User Request
   ↓
K3s Service (NodePort)
   ↓
Rust Host Pod
   ↓
Retriever (NFS artifacts)
   ↓
llama.cpp (/v1/chat/completions)
   ↓
Final Response
```

---

## Offline Flow

```text
Trigger (manual / future API)
   ↓
Slurm Job
   ↓
RAG Indexer
   ↓
Artifacts written to NFS
   ↓
K3s App reads updated artifacts
```

---

# ⚙️ LLM Backend Evolution

## Phase 1 (previous)

* Ollama for:

  * chat
  * embeddings

## Phase 2 (current)

* llama.cpp replaces Ollama for:

  * chat inference ✔
* Ollama still optional

## Phase 3 (planned)

* llama.cpp for:

  * embeddings
  * LoRA adapters
* remove Ollama completely

---

# ✅ Current Working State

* Slurm cluster ✔
* GPU scheduling ✔
* NFS shared storage ✔
* K3s cluster ✔
* llama.cpp deployed ✔
* OpenAI-compatible API ✔
* Rust app connected to llama.cpp ✔
* MCP tools loaded ✔
* RAG working from NFS ✔
* warmup job implemented ✔

---

# ⚠️ Known Limitations

## 1. Tool usage

* tools load correctly
* LLM may skip tool calls

Issues:

* direct answering
* incomplete reasoning
* tool avoidance

Planned:

* tool forcing
* retry loop
* stricter routing rules

---

## 2. JSON output reliability

* model may ignore JSON instructions

Planned:

* stronger system prompt
* response validation + retry

---

## 3. Retrieval grounding

* model may hallucinate beyond context

Planned:

* stricter prompt constraints
* "answer ONLY from context" enforcement

---

## 4. Cold start latency

* first request is slow (model warmup)

Mitigation:

* warmup Kubernetes Job ✔
* prompt caching ✔

---

# 🧠 Retrieval Strategy

Current:

* JSON artifacts
* local embeddings
* in-memory scoring

Advantages:

* simple
* transparent
* debuggable

Future:

* llama.cpp embeddings
* optional vector DB (Qdrant)

---

# 🖥 Hardware Topology

## Laptop

* development
* Docker buildx
* deployment scripts
* curl client

## Raspberry

* K3s control-plane
* Slurm controller
* NFS client

## Jetson

* GPU inference (llama.cpp)
* Slurm worker
* NFS server
* K3s worker

---

# 🚀 Roadmap

## Completed

* Slurm + GPU
* NFS setup
* K3s cluster
* llama.cpp runtime container
* OpenAI-compatible API
* RAG integration
* warmup job

## Next

* llama.cpp embeddings
* LoRA support
* tool reliability improvements
* automated Slurm pipelines
* UI layer

---

# 🧠 Design Principles

* separate online / offline workloads
* keep system observable
* prefer simple formats first
* avoid unnecessary abstractions
* use OpenAI-compatible APIs as standard
* build incrementally

---

# 📌 Summary

You now have a fully working **mini AI infrastructure platform**:

* distributed compute (Slurm)
* shared storage (NFS)
* container orchestration (K3s)
* GPU inference (llama.cpp)
* OpenAI-compatible serving
* RAG pipeline (end-to-end)

Next milestone:

👉 Fully local pipeline (llama.cpp for chat + embeddings + LoRA)
