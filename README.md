# 🧠 AI Platform Lab (Rust + MCP + RAG + Ollama)

A minimal but production-structured **AI platform and infra lab** built primarily in Rust.

This repository currently contains:

- a modular Rust AI orchestration platform
- MCP and HTTP tool integration
- RAG with markdown and embeddings-based retrieval
- Docker and local execution modes
- the start of an infra-focused roadmap around Slurm, Ansible, Apptainer, and later k3s

---

# 🎯 Project Goals

This is not just a demo chatbot.

The goal is to build and explore:

> A modular AI platform where LLM backends, retrieval backends, and tool systems are swappable,

while also using it as a hands-on lab for:

- AI infrastructure
- model serving and orchestration
- retrieval pipelines
- job scheduling
- lightweight cluster operations
- deployment patterns on personal hardware

Target hardware for later infra experiments:

- laptop
- Raspberry Pi 4
- Jetson Orin Nano

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
        │      ├── in-memory markdown retriever
        │      └── embeddings-based retriever
        └──→ ToolProvider
               ├── HTTP tools server
               └── MCP tools server (stdio, child process)
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
  → Retrieval (when needed)
  → LLM Response
    → ToolCall?
       → Execute Tool
       → Feed Result Back to LLM
  → Final Answer
```

## Tool Abstraction

```rust
trait ToolProvider {
  async fn list_tools(...);
  async fn call_tool(...);
}
```

## LLM Backend

```rust
trait LlmBackend {
  async fn chat(...);
}
```

## Retrieval

```rust
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

# 🧩 Current RAG State

- markdown knowledge base under:
  - knowledge_base/hr
  - knowledge_base/engineering
  - knowledge_base/product
- offline indexer
- chunk generation
- embeddings generation via Ollama
- persisted retrieval artifacts

```text
artifacts/rag/
  chunks.json
  embeddings.json
  manifest.json
```

Retrieval modes:

- noop
- in-memory markdown lexical retriever
- embeddings-based retriever (local JSON artifacts)

No vector database yet by design.

---

# 🔀 Query Routing

- tool_first
- retrieval_first
- hybrid

---

# 📊 Example Response

```json
{
  "answer": "...",
  "steps": [...],
  "sources": [...],
  "retrieval_confidence": 0.4,
  "safety_notes": [...]
}
```

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
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

---

# 🔥 Achievements

- clean Rust architecture
- async orchestration loop
- MCP integration
- RAG with offline indexing
- swappable system design

---

# 🚀 Next Steps

- embeddings improvements
- vLLM backend
- Slurm integration

---

# 📁 Infra Layout

```text
infra/
  ansible/
  slurm/
  apptainer/
  notes/
```

---

# 🧩 Notes

- MCP requires clean stdout
- logs → stderr
- no vector DB yet
- no auth yet
