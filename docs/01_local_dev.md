<!--
AI Platform Lab Documentation
Standardized Edition
Environment:
- Laptop: development machine
- Raspberry Pi: Slurm controller + K3s control-plane
- Jetson Orin Nano: GPU worker + inference + training node
- External Tailscale endpoint: 100.109.72.92
-->

# 🧪 Local Development (Rust + Tools + Ollama + RAG)

This guide walks through running the AI platform **locally**, without Kubernetes or Slurm.

You will:

* run the tools server
* run the Rust host
* connect to an LLM (Ollama)
* test chat, tools, and retrieval (RAG)

---

# 🎯 Goal

By the end of this guide, you should be able to:

* call `/chat`
* trigger tool execution
* retrieve knowledge from local RAG artifacts

---

# 1️⃣ Run Tools Server

Start the tools server:

```bash
cargo run -p tools-server
```

Verify it is working:

```bash
curl http://localhost:3001/health
curl http://localhost:3001/tools
```

Expected:

* `/health` returns OK
* `/tools` lists available tools (e.g. weather)

---

# 2️⃣ Run Host (without LLM yet)

Open two terminals:

### Terminal 1

```bash
cargo run -p tools-server
```

### Terminal 2

```bash
cargo run -p host
```

At this stage, host runs but LLM calls will fail (no backend yet).

---

# 3️⃣ Start LLM (Ollama)

Run Ollama:

```bash
docker run -d -p 11434:11434 ollama/ollama:0.18.2
```

Pull model:

```bash
docker exec -it <container_id> ollama pull llama3
```

---

# 4️⃣ Run Host with LLM

```bash
cargo run -p host
```

---

# 5️⃣ Test Chat (Tool Call)

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the weather in Berlin?"}'
```

Expected:

* tool is called (weather)
* final answer returned

---

# 6️⃣ Run Full Stack with Docker Compose

Start everything:

```bash
docker compose up --build
```

Pull model inside container:

```bash
docker exec -it ai_platform_ollama ollama pull llama3
```

Check available models:

```bash
docker exec -it ai_platform_ollama ollama list
```

---

# 7️⃣ Test Chat via Docker

### Tool-first query

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the weather in Berlin?"}'
```

### Retrieval-style query (no RAG yet)

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

---

# 8️⃣ Enable HTTP Tool Provider (optional)

```bash
TOOL_PROVIDER=http docker compose --profile http-tools up --build
```

Test again:

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the weather in Berlin?"}'
```

---

# 9️⃣ Enable RAG (Embeddings)

Pull embedding model:

```bash
docker exec -it ai_platform_ollama ollama pull nomic-embed-text
```

Run indexer:

```bash
cargo run -p rag_indexer
```

This generates:

```text
artifacts/rag/
  chunks.json
  embeddings.json
  manifest.json
```

---

# 🔟 Run Host with Retrieval Enabled

```bash
RETRIEVAL_BACKEND=embeddings_local \
RAG_ARTIFACTS_PATH=artifacts/rag \
cargo run -p host
```

---

# 🧪 Test Scenarios

## 🔧 Tool-first

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the weather in Berlin?"}'
```

Expected:

* tool is used
* no retrieval

---

## 📚 Retrieval-first

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

Expected:

* RAG context used
* sources returned

---

## 🔀 Hybrid Query

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the expense policy and what is the current status of my reimbursement?"}'
```

Expected:

* retrieval used for policy
* tool used for live status (if available)

---

# ⚠️ Notes

* Tool usage is not always reliable (LLM may skip tools)
* JSON responses may break (model formatting issues)
* First responses can be slower (model warmup)

---

# 📌 Summary

You now have a fully working **local AI platform**:

* Rust orchestration layer
* tool execution (MCP / HTTP)
* Ollama LLM backend
* RAG pipeline with embeddings
* end-to-end chat flow

Next step:

👉 move to containerized + GPU-backed inference (llama.cpp)
