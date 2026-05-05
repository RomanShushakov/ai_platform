# 🧠 AI Platform Lab (Rust + MCP + RAG + Slurm + K3s + llama.cpp)

A minimal but **production-structured AI platform and infrastructure lab** built primarily in Rust.

This repository combines:

* a modular Rust AI orchestration platform
* MCP and HTTP tool integration
* RAG with markdown + embeddings-based retrieval
* GPU-backed LLM inference via llama.cpp
* a working Slurm cluster (CPU + GPU)
* NFS shared storage between nodes
* a K3s cluster for deployment

---

# 🎯 Project Goals

This is not just a demo chatbot.

The goal is to build:

> A modular AI platform where LLM backends, retrieval systems, and tool providers are fully swappable

while also serving as a hands-on lab for:

* AI infrastructure
* model orchestration
* retrieval pipelines (RAG)
* batch compute (Slurm)
* shared storage (NFS)
* container orchestration (K3s)
* GPU inference on edge hardware

Target hardware:

* 💻 Laptop (dev + build + control)
* 🍓 Raspberry Pi 4 (control-plane)
* ⚡ Jetson Orin Nano (GPU worker)

---

# 🏗 Architecture Overview

```text
Client (curl / UI)
        ↓
K3s Cluster (Raspberry + Jetson)
        ↓
Rust Host (AI Orchestrator)
        ├──→ LLM Backend (llama.cpp, OpenAI-compatible)
        ├──→ Retriever (RAG from NFS)
        └──→ Tool Providers (MCP / HTTP)

Offline Plane:
        Slurm (Raspberry → Jetson)
                ↓
            NFS Storage
                ↓
        App consumes artifacts
```

---

# 🔁 LLM Backend Evolution

## Phase 1

* Ollama (chat + embeddings)

## Phase 2 (current)

* llama.cpp for chat inference ✔
* OpenAI-compatible API ✔
* Ollama optional

## Phase 3 (planned)

* llama.cpp embeddings
* LoRA adapters
* remove Ollama entirely

---

# 📦 Workspace Structure

```text
workspace/
  host/              # Rust AI orchestrator
  tools_server/      # MCP / HTTP tools
  llm_client/        # LLM abstraction layer
  shared_types/      # shared domain models
  knowledge_base/    # markdown KB
  indexer/           # RAG indexing
  artifacts/         # generated RAG data
  infra/             # Slurm + K3s + NFS
```

---

# 🧠 Core Concepts

## LLM Loop

```text
User Input
  → Query Routing
  → Retrieval (optional)
  → LLM Response
    → ToolCall?
       → Execute Tool
       → Feed Result Back
  → Final Answer
```

---

# 🧩 RAG State

Current implementation:

* markdown knowledge base
* offline indexing
* embeddings (initially via Ollama)
* JSON artifacts (no vector DB)

```text
artifacts/rag/
  chunks.json
  embeddings.json
  manifest.json
```

✔ intentionally simple
✔ fully debuggable
✔ no external dependencies

---

# 🏗 Infra Overview

```text
infra/
  ansible/
  slurm/
  k3s/
  images/
  notes/
```

---

# 🖥 Current Infra State

## Slurm

* controller: Raspberry
* worker: Jetson
* GPU scheduling (GRES)

## NFS

* server: Jetson
* client: Raspberry + K3s pods
* shared path:

```
/home/roman/nfs
```

## K3s

* Raspberry → control-plane
* Jetson → GPU worker

Running workloads:

* Rust host
* llama.cpp server
* Ollama (optional)
* warmup jobs

---

# 📦 Storage Layout

```text
/home/roman/nfs/
├── slurm/
├── rag/
│   ├── knowledge_base/
│   └── artifacts/
├── models/
│   └── gguf/
└── logs/
```

---

# 🔥 What Works End-to-End

✔ Slurm jobs run on Jetson
✔ Outputs shared via NFS
✔ K3s runs distributed workloads
✔ llama.cpp serving via OpenAI API
✔ Rust app integrated with llama.cpp
✔ RAG working from NFS
✔ MCP tools integrated
✔ Warmup job eliminates cold start

---

# 🧭 Architecture Split

## 🟢 Online (Serving)

```text
User → K3s → Rust Host → RAG / Tools → llama.cpp → Response
```

## 🔵 Offline (Batch)

```text
Trigger → Slurm → NFS → Rust Host consumes artifacts
```

---

# ⚠️ Known Limitations

## LLM behavior

* sometimes ignores tool usage
* may hallucinate beyond retrieved context

## JSON reliability

* model may not follow strict JSON schema

## Cold start

* first request slow (mitigated via warmup)

---

# 🚀 Roadmap

## Completed

* Slurm cluster ✔
* GPU scheduling ✔
* NFS ✔
* K3s cluster ✔
* Rust app deployed ✔
* llama.cpp integration ✔
* OpenAI-compatible serving ✔
* RAG working ✔

## Next

* llama.cpp embeddings
* LoRA support
* tool reliability improvements
* automated Slurm pipelines
* UI layer

---

# 🧠 Design Principles

* separate online vs offline workloads
* keep system observable
* prefer simple formats first
* avoid unnecessary infrastructure
* use OpenAI-compatible APIs as standard
* build incrementally on real hardware

---

# 📌 Summary

You now have a fully working **mini AI platform + infra lab**:

* distributed compute → Slurm
* shared storage → NFS
* orchestration → K3s
* GPU inference → llama.cpp
* modular orchestration → Rust host
* retrieval pipeline → RAG (JSON artifacts)

Next milestone:

👉 fully local pipeline (llama.cpp for chat + embeddings + LoRA)
