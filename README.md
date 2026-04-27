# 🧠 AI Platform Lab (Rust + MCP + RAG + Slurm + K3s)

A minimal but production-structured **AI platform and infrastructure lab** built primarily in Rust.

This repository combines:

- a modular Rust AI orchestration platform
- MCP and HTTP tool integration
- RAG with markdown + embeddings-based retrieval
- a working Slurm cluster (CPU + GPU)
- NFS shared storage between nodes
- a working K3s cluster for deployment

---

# 🎯 Project Goals

This is not just a demo chatbot.

The goal is to build:

> A modular AI platform where LLM backends, retrieval systems, and tool providers are fully swappable

while also serving as a hands-on lab for:

- AI infrastructure
- model orchestration
- retrieval pipelines
- batch compute (Slurm)
- shared storage (NFS)
- container orchestration (K3s)
- deployment on real hardware

Target hardware:

- Laptop (dev + control)
- Raspberry Pi 4 (controller + K3s control plane)
- Jetson Orin Nano (GPU worker + K3s worker)

---

# 🏗 Architecture Overview

```text
Client (curl / UI)
        ↓
K3s (Raspberry + Jetson)
        ↓
Rust Host (orchestrator)
        ├──→ LLM Backend (Ollama)
        ├──→ Retriever (JSON artifacts from NFS)
        └──→ MCP Tool Server (embedded)

Offline Plane:
        Slurm (Raspberry → Jetson)
                ↓
            NFS Storage
                ↓
        App reads results
```

---

# 📦 Workspace Structure

```text
workspace/
  host/
  tools_server/
  llm_client/
  shared_types/
  knowledge_base/
  indexer/
  artifacts/
  infra/
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

- markdown knowledge base
- offline indexing
- embeddings via Ollama
- JSON artifacts

```text
artifacts/rag/
  chunks.json
  embeddings.json
  manifest.json
```

✔ No vector DB (intentional)

---

# 🏗 Infra Overview

```text
infra/
  ansible/
  slurm/
  k3s/
  notes/
```

---

# 🖥 Current Infra State

## Slurm
- controller: Raspberry
- worker: Jetson
- GPU scheduling (GRES)

## NFS
- server: Jetson
- client: Raspberry
- shared path:

```
/home/roman/nfs
```

## K3s
- Raspberry = control-plane
- Jetson = worker
- workloads distributed across nodes

---

# 📦 Storage Layout

```text
/home/roman/nfs/
├── slurm/
├── rag/
├── models/
└── logs/
```

---

# 🔥 What Works End-to-End

✔ Slurm jobs run on Jetson  
✔ Outputs visible via NFS  
✔ K3s runs distributed pods  
✔ Rust app deployed in cluster  
✔ Ollama running in cluster  
✔ RAG working from NFS  

---

# 🧭 Architecture Split

## Online (Serving)

User → K3s → Rust → RAG / Tools / LLM → response

## Offline (Batch)

Trigger → Slurm → NFS → app consumes results

---

# 🚀 Roadmap

## Done

- Slurm cluster ✔
- GPU scheduling ✔
- NFS ✔
- K3s cluster ✔
- Rust app deployed ✔
- Ollama integrated ✔
- RAG working ✔

## Next

- Slurm-based RAG rebuild jobs
- automate pipelines
- improve tool usage (tool forcing)
- split services (indexer, training)
- add UI
- experiment with vLLM

---

# 🧠 Design Principles

- separate online vs offline
- simple RAG first
- real infra over mocks
- reproducible via Ansible
- incremental complexity

---

# 📌 Summary

You now have a fully working **mini AI platform + infra lab**:

- distributed compute (Slurm)
- shared storage (NFS)
- orchestration (K3s)
- model serving (Ollama)
- RAG pipeline (JSON artifacts)

Next step:

👉 automate batch pipelines via Slurm
