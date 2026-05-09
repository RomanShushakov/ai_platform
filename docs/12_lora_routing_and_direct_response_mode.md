# 🚀 12. LoRA Routing and Direct Response Mode

This step integrates the trained LoRA runtime into `ai-platform-host`.

The goal is **not** to replace the base model.

Instead:

* base model remains responsible for:

  * RAG
  * tools
  * MCP-style orchestration
  * structured JSON/tool-call loops

* LoRA model becomes:

  * a specialized behavior/style endpoint
  * useful for:

    * lab-style formatting
    * controlled answer styles
    * future JSON discipline tuning
    * future tool-call specialization

---

# 🎯 Goal

After this step, you will have:

* multiple llama.cpp runtimes behind K3s
* separate base vs LoRA backends
* configurable routing inside `ai-platform-host`
* direct response mode
* preserved RAG/tool architecture
* explicit backend selection from API requests

---

# 🧭 Final Runtime Architecture

```text
                    ┌────────────────────┐
                    │ ai-platform-host   │
                    │ Rust orchestration │
                    └─────────┬──────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼

┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│ llama-cpp      │  │ llama-cpp-lora │  │ llama-cpp-embed│
│ port 8000      │  │ port 8003      │  │ port 8001      │
│ base model     │  │ base+LoRA      │  │ embeddings     │
└────────────────┘  └────────────────┘  └────────────────┘
```

---

# 🧠 Design Decision

We intentionally keep:

```text
agent/tool/RAG mode
```

separate from:

```text
direct LoRA generation mode
```

because the LoRA adapter was trained on:

```text
user -> assistant style answers
```

not on:

```text
JSON tool-call protocol
```

This separation keeps the platform stable.

---

# 📦 Runtime Profiles

Current runtime configuration:

| Profile | Backend        | Purpose                     |
| ------- | -------------- | --------------------------- |
| default | llama-cpp      | RAG + tools + orchestration |
| lora    | llama-cpp-lora | direct lab-style responses  |

---

# 🧩 Request Routing

Two request controls now exist:

| Field           | Purpose                        |
| --------------- | ------------------------------ |
| `llm_profile`   | chooses backend                |
| `response_mode` | chooses orchestration behavior |

---

# 🧠 Current Supported Modes

## Agent mode (default)

Uses:

* tools
* retrieval
* orchestration loop
* structured responses

Example:

```json
{
  "message": "How do I request vacation?"
}
```

---

## Direct mode

Skips:

* tools
* retrieval
* orchestration loop

Calls backend directly.

Useful for:

* LoRA behavior testing
* style adapters
* deterministic formatting

Example:

```json
{
  "message": "What is Slurm?",
  "llm_profile": "lora",
  "response_mode": "direct"
}
```

---

# 📁 Files Updated

## `host/src/api.rs`

Added request field:

```rust
pub response_mode: Option<String>,
```

Added routing:

```rust
match response_mode {
    "agent" => { ... }
    "direct" => { ... }
}
```

---

## `host/src/domain/llm_loop.rs`

Added:

```rust
run_direct_chat(...)
```

This path:

* skips retrieval
* skips tools
* directly calls selected backend

---

## `host/src/domain/llm_backend.rs`

Added:

```rust
async fn chat_direct(...)
```

---

## `host/src/adapters/openai_compat_llm_backend.rs`

Added direct raw completion support.

---

## OpenAI-compatible client

Added raw text extraction from:

```text
/v1/chat/completions
```

without tool-loop parsing.

---

# 🛠 Build and Deploy

## on laptop:

```bash
cargo check

./scripts/build_push_ai_platform_host_arm64.sh

./scripts/sync_k3s_manifests.sh
```

---

## on raspberry:

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/ai-platform-host.yml

kubectl rollout restart deployment/ai-platform-host -n ai-platform

kubectl rollout status deployment/ai-platform-host -n ai-platform

kubectl logs -n ai-platform deployment/ai-platform-host
```

---

# 🧪 Test Base Agent Mode

## on laptop:

```bash
curl -X POST http://100.109.72.92:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

Expected behavior:

* retrieval enabled
* RAG answer generated
* sources returned

---

# 🧪 Test Tool Routing

## on laptop:

```bash
curl -X POST http://100.109.72.92:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the weather in Berlin?"}'
```

Expected behavior:

* tool-first route
* weather tool used
* no retrieval required

---

# 🧪 Test LoRA Direct Mode

## on laptop:

```bash
curl -X POST http://100.109.72.92:30080/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is Slurm?",
    "llm_profile": "lora",
    "response_mode": "direct"
  }'
```

Expected response:

```text
Lab note: Slurm is a workload manager used to schedule batch jobs on compute nodes.
```

---

# 🧪 Additional Validation

## Kubernetes

```bash
kubectl get pods -n ai-platform -o wide
kubectl get svc -n ai-platform
```

---

## LoRA endpoint directly

```bash
curl -X POST http://jetson:8003/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model":"qwen-lora-gpu",
    "messages":[
      {
        "role":"user",
        "content":"What is Kubernetes?"
      }
    ],
    "temperature":0,
    "max_tokens":48
  }'
```

---

# 🧠 Important Interpretation

The LoRA adapter should currently be treated as:

```text
behavior/style specialization
```

not as:

```text
authoritative factual memory
```

Factual grounding should still come from:

* RAG
* tools
* structured retrieval

---

# 🔥 Operational Notes

## GPU contention

Jetson Orin Nano has limited VRAM and RAM.

Before GPU LoRA training:

## on raspberry:

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

# 📌 Final Result

You now have:

* GPU-backed llama.cpp inference ✔
* embedding runtime ✔
* RAG indexing ✔
* Slurm orchestration ✔
* Apptainer jobs ✔
* CPU LoRA training ✔
* GPU LoRA training ✔
* LoRA GGUF conversion ✔
* K3s multi-runtime deployment ✔
* Rust orchestration host ✔
* profile-based LLM routing ✔
* direct LoRA response mode ✔

---

# 🏁 Project Status

At this point, the platform demonstrates a complete miniature AI infrastructure stack:

```text
training
→ conversion
→ registry
→ Slurm jobs
→ GPU execution
→ K3s deployment
→ RAG
→ inference routing
→ Rust orchestration
→ LoRA specialization
```

on heterogeneous ARM infrastructure:

```text
Laptop
→ Raspberry Pi
→ Jetson Orin Nano
```

This is already a strong practical GPU/AI infrastructure lab.
