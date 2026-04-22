# 🧠 AI Platform Lab (Rust + MCP + RAG + Slurm + K3s)

A minimal but production-structured **AI platform and infrastructure lab** built primarily in Rust.

This repository combines:

- a modular Rust AI orchestration platform
- MCP and HTTP tool integration
- RAG with markdown + embeddings-based retrieval
- a working Slurm cluster (CPU + GPU)
- NFS shared storage between nodes
- a roadmap toward K3s deployment

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
- Raspberry Pi 4 (controller)
- Jetson Orin Nano (GPU worker)

---

# 🏗 Architecture Overview

```text
Client (curl / UI)
        ↓
Rust Host (orchestrator)
        ├──→ LLM Backend
        │      ├── Ollama
        │      └── vLLM (planned)
        ├──→ Retriever
        │      ├── noop
        │      ├── markdown
        │      └── embeddings (JSON artifacts)
        └──→ ToolProvider
               ├── HTTP tools server
               └── MCP tools server

Offline / Batch Plane:
        Slurm Cluster (Raspberry → Jetson)
                 ↓
            NFS Storage
                 ↓
          App / K3s (future)
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

## Traits

```rust
trait ToolProvider {
  async fn list_tools(...);
  async fn call_tool(...);
}

trait LlmBackend {
  async fn chat(...);
}

trait Retriever {
  async fn retrieve(...);
}
```

---

# 📚 Knowledge Base

```text
knowledge_base/
  hr/
  engineering/
  product/
```

---

# 🧩 RAG State

- markdown-based KB
- offline indexing
- embeddings via Ollama
- JSON artifacts

```text
artifacts/rag/
  chunks.json
  embeddings.json
  manifest.json
```

No vector DB (by design for now).

---

# 🔀 Query Routing

- tool_first
- retrieval_first
- hybrid

---

# ⚙️ Configuration

```env
TOOL_PROVIDER=mcp | http
LLM_BACKEND=ollama | vllm
RETRIEVAL_BACKEND=noop | inmemory_markdown | embeddings
```

---

# ▶️ Run

```bash
cargo run -p host
```

```bash
docker compose up --build
```

---

# 🧪 Example Request

```bash
curl -X POST http://localhost:3000/chat   -H "Content-Type: application/json"   -d '{"message":"How do I request vacation?"}'
```

---

# 🏗 Infra Overview

```text
infra/
  ansible/
  slurm/
  apptainer/
  notes/
```

---

# 🖥 Current Infra State

## Slurm

- built from source
- controller: Raspberry
- worker: Jetson
- GPU scheduling via GRES

## NFS

- server: Jetson
- client: Raspberry
- shared path:

```
/home/roman/nfs
```

## Verified

- jobs run on Jetson
- outputs visible via NFS on Raspberry
- GPU jobs execute correctly

---

# 🧭 Architecture Split

## Online (App)

Rust → tools / RAG → response

## Offline (Batch)

Trigger → Slurm → NFS → app consumes

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

# 🚀 Roadmap

1. Slurm + GPU ✅
2. NFS shared storage ✅
3. K3s cluster (next)
4. Deploy Rust app to K3s
5. Mount NFS into pods
6. Optional: Qdrant

---

# 🧠 Design Principles

- separation of online vs batch
- simple RAG first
- real infra over mocks
- reproducible via Ansible
- avoid overengineering

---

# 🔥 Achievements

- modular Rust AI platform
- MCP integration
- RAG pipeline
- working Slurm cluster
- GPU scheduling
- shared storage (NFS)
- real distributed execution

---

# 📌 Notes

- MCP requires clean stdout
- logs → stderr
- no auth yet
- no vector DB yet (intentional)
