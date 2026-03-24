# 🧠 AI Platform (Rust + MCP + Ollama)

A minimal but **production-structured AI orchestration platform** built in Rust.

This project demonstrates how to:

* Orchestrate LLM interactions
* Dynamically select and execute tools
* Abstract tool transport (HTTP ↔ MCP)
* Run a local LLM (Ollama)
* Build a clean, extensible backend architecture

---

# 🎯 Project Goals

This is **not just a demo chatbot**.

Instead, the goal is to build:

> A modular AI platform where LLM backends and tool systems are **fully swappable**

---

# 🏗 Architecture Overview

## Current Architecture

```
Client (curl / UI)
        ↓
Rust Host (orchestrator)
        ├──→ LLM (Ollama)
        └──→ ToolProvider
                 ├── HTTP tools server
                 └── MCP tools server (stdio, child process)
```

---

# 📦 Workspace Structure

```
workspace/
  host/            # Main orchestrator service
  tools_server/    # Tools implementation (HTTP + MCP)
  llm_client/      # LLM abstraction (Ollama for now)
  shared_types/    # Shared DTOs
```

---

# 🧠 Core Concepts

## 1. LLM Loop (Orchestration Engine)

Flow:

```
User Input
  → LLM Response
    → ToolCall?
       → Execute Tool
       → Feed Result Back to LLM
  → Final Answer
```

Features:

* Multi-step reasoning
* Tool chaining
* Max step protection
* Structured logging

---

## 2. Tool Abstraction

We introduced:

```rust
trait ToolProvider {
    async fn list_tools(...)
    async fn call_tool(...)
}
```

### Implementations:

| Provider | Description                           |
| -------- | ------------------------------------- |
| HTTP     | Calls tools via REST                  |
| MCP      | Uses Model Context Protocol via stdio |

---

## 3. MCP Integration (Model Context Protocol)

We implemented:

* ✅ MCP server in Rust (`tools-server`)
* ✅ MCP client in host
* ✅ stdio transport via child process

### Why MCP?

* Standardized tool interface
* No HTTP overhead
* Better for local / secure / composable systems

---

## 4. LLM Integration (Ollama)

* Local model execution
* Uses `/api/chat`
* JSON-based structured responses
* Tool-aware prompting

---

# ⚙️ Configuration

Environment variables control behavior.

## Tool Provider

```
TOOL_PROVIDER=mcp | http
```

## MCP Mode

```
MCP_TOOLS_BINARY=/usr/local/bin/tools-server
```

## HTTP Mode

```
TOOLS_BASE_URL=http://tools-server:3001
```

## LLM

```
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama3
```

---

# 🐳 Docker Setup

Single `docker-compose.yml` supports both modes.

## Run MCP mode (default)

```
docker compose up --build
```

* Host spawns tools-server internally
* No tools-server container needed

---

## Run HTTP mode

```
TOOL_PROVIDER=http docker compose --profile http-tools up --build
```

* Runs tools-server as separate container
* Host communicates via HTTP

---

# 🧪 Example Request

```
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

Example response:

```json
{
  "answer": "Employees can request vacation in the HR portal...",
  "steps": [
    "Loaded 2 tool(s)",
    "LLM requested tool 'search_docs'",
    "Executed tool 'search_docs'",
    "Generated final answer"
  ]
}
```

---

# 🔥 What We Achieved

✅ Clean Rust architecture
✅ Async orchestration loop
✅ Tool abstraction layer
✅ MCP protocol integration
✅ Dockerized local AI stack
✅ Config-driven backend switching

---

# 🧠 Key Design Insights

### 1. LLM is NOT the system

The system is:

```
LLM + Orchestration + Tools + Infra
```

---

### 2. MCP replaces ad-hoc integrations

Instead of:

```
LLM → random APIs
```

We now have:

```
LLM → Host → MCP → Tools
```

---

### 3. Separation of concerns

| Layer        | Responsibility |
| ------------ | -------------- |
| Host         | orchestration  |
| LLM client   | inference      |
| ToolProvider | execution      |
| MCP server   | tool exposure  |

---

# 🚀 Next Steps

## 1. LLM Backend Abstraction

Introduce:

```
LLM_BACKEND=ollama | vllm
```

* Add `trait LlmClient`
* Implement `vLLM` client
* Make backend configurable

---

## 2. Slurm Integration

Target architecture:

```
Host → vLLM (Slurm GPU job)
```

Goals:

* GPU-backed inference
* Remote endpoint
* HPC awareness

---

## 3. Improvements

* Streaming responses
* Tool caching
* Observability (metrics)
* Better tool schemas
* Multi-tool planning

---

# 🧩 Notes

* MCP uses **stdio transport**, so stdout must be clean (no logs)
* Logs must go to **stderr**
* Host spawns tools-server automatically in MCP mode

---

# 🎯 Final Thought

You now have:

> A minimal AI platform with **real architectural foundations**

Not a toy.

This is the same direction used in:

* internal AI platforms
* copilots
* enterprise knowledge systems

---

Tomorrow → we make LLM backend swappable and move toward GPU / Slurm 🚀
