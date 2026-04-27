# 🖥 Infra Track: Slurm + NFS + K3s (Full Lab)

This directory tracks the infrastructure side of the AI platform project.

The goal is not just to run a chatbot, but to understand how:

- online serving
- batch compute
- shared storage
- container orchestration

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
│ Rust Host    │         │   Ollama     │
│ (API + MCP)  │         │ (LLM + Embed)│
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

- chat API
- tool orchestration (MCP / HTTP)
- retrieval (RAG)
- LLM interaction

Runs inside K3s.

---

## 2. Batch Plane (Slurm)

Handles:

- RAG artifact generation
- embedding jobs
- training (future)
- preprocessing

Runs on Jetson (GPU capable).

---

## 3. Storage Plane (NFS)

Shared directory:

```
/home/roman/nfs
```

Used for:

- RAG artifacts
- job outputs
- models
- logs

Mounted:

```
/mnt/nfs
```

inside K3s pods.

---

## 4. Orchestration Plane (K3s)

Cluster:

- Raspberry → control-plane
- Jetson → worker

Runs:

- Rust host
- Ollama
- future services

---

# 📦 Storage Layout

```
/home/roman/nfs/
├── slurm/
│   └── job-examples/
├── rag/
│   └── artifacts/
│       ├── chunks.json
│       ├── embeddings.json
│       └── manifest.json
├── models/
└── logs/
```

---

# 🔥 End-to-End Data Flow

## Online Flow

```text
User Request
   ↓
K3s Service (NodePort)
   ↓
Rust Host Pod
   ↓
Retriever (JSON artifacts from NFS)
   ↓
Ollama (LLM + embeddings)
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

# ✅ Current Working State

- Slurm cluster ✔
- GPU scheduling ✔
- NFS shared storage ✔
- K3s cluster ✔
- Ollama deployed ✔
- Rust app deployed ✔
- MCP tools loaded ✔
- RAG working from NFS ✔

---

# ⚠️ Known Limitations

## Tool usage (MCP)

- tools load correctly
- LLM does not reliably call tools

Observed issues:

- direct answering
- invalid JSON (markdown wrapping)
- hallucinated tool calls

Planned solution:

- tool forcing
- retry loop
- stricter JSON parsing

---

# 🧠 Retrieval Strategy

Current:

- JSON artifacts
- embeddings via Ollama

Advantages:

- simple
- debuggable
- minimal infra

Future:

- optional vector DB (Qdrant)

---

# 🖥 Hardware Topology

## Laptop
- development
- Ansible
- curl client

## Raspberry
- Slurm controller
- K3s control-plane
- NFS client

## Jetson
- Slurm worker
- GPU node
- NFS server
- K3s worker

---

# 🚀 Roadmap

## Completed

- Slurm + GPU
- NFS setup
- K3s cluster
- app deployment
- RAG integration

## Next

- Slurm-based RAG rebuild
- automate pipelines
- improve tool usage
- add UI
- experiment with vLLM

---

# 🧠 Design Principles

- separate online/offline workloads
- keep system observable
- prefer simple formats first
- avoid unnecessary complexity
- build incrementally

---

# 📌 Summary

You now have a fully working **mini AI infrastructure platform**:

- distributed compute (Slurm)
- shared storage (NFS)
- container orchestration (K3s)
- model serving (Ollama)
- RAG pipeline (end-to-end)

Next milestone:

👉 Slurm-driven automated RAG pipeline
