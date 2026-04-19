# 🧠 AI Platform (Rust + MCP + RAG + Ollama)

A minimal but **production-structured AI orchestration platform** built in Rust.

This project demonstrates how to:

- Orchestrate LLM interactions
- Dynamically select and execute tools
- Abstract tool transport (HTTP ↔ MCP)
- Abstract LLM backends (Ollama ↔ future vLLM)
- Ground answers with retrieval (RAG)
- Build a clean, extensible backend architecture

---

# 🎯 Project Goals

This is **not just a demo chatbot**.

Instead, the goal is to build:

> A modular AI platform where LLM backends, retrieval backends, and tool systems are **fully swappable**

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
        │      └── embeddings-based retriever (planned)
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
```

---

# 🧠 Core Concepts

## LLM Loop

```text
User Input
  → Retrieval
  → LLM Response
    → ToolCall?
       → Execute Tool
       → Feed Result Back to LLM
  → Final Answer
```

---

## Tool Abstraction

```rust
trait ToolProvider {
    async fn list_tools(...)
    async fn call_tool(...)
}
```

---

## LLM Backend

```rust
trait LlmBackend {
    async fn chat(...)
}
```

---

## Retrieval

```rust
trait Retriever {
    async fn retrieve(...)
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

# 📊 Example Response

```json
{
  "answer": "...",
  "steps": [...],
  "sources": [...],
  "retrieval_confidence": 0.4
}
```

---

# ⚙️ Configuration

```env
TOOL_PROVIDER=mcp | http
LLM_BACKEND=ollama | vllm
RETRIEVAL_BACKEND=noop | inmemory_markdown
```

---

# 🐳 Docker

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

# 🔥 What We Achieved

- Clean Rust architecture
- Async orchestration loop
- Tool abstraction layer
- MCP integration
- RAG support

---

# 🚀 Next Steps

- Embeddings-based retrieval
- vLLM backend
- Slurm integration

---

# 🧩 Notes

- MCP requires clean stdout
- Logs must go to stderr
- Current RAG is lexical
