<!--
AI Platform Lab Documentation
Standardized Edition
Environment:
- Laptop: development machine
- Raspberry Pi: Slurm controller + K3s control-plane
- Jetson Orin Nano: GPU worker + inference + training node
- External Tailscale endpoint: 100.109.72.92
-->

# 🤖 AI Platform Deployment on K3s (Host + Ollama)

This step deploys the **online serving layer** of the platform into K3s.

Components:

* Rust **AI Host** (orchestrator)
* **Ollama** (LLM + embeddings)
* RAG artifacts (via NFS)

---

# 🎯 Goal

After this step, you will have:

* chat API running in K3s
* LLM backend (Ollama) connected
* working `/chat` endpoint
* ability to test tool + retrieval flows

---

# 🧭 Architecture (Online Path)

```text
Client (curl)
   ↓
K3s NodePort
   ↓
AI Platform Host (Rust)
   ↓
   ├── Ollama (LLM / embeddings)
   ├── Tools (HTTP / MCP)
   └── RAG (NFS artifacts)
```

---

# 1️⃣ Build & Push AI Host (ARM64)

From laptop:

```bash
./scripts/build_push_ai_platform_host_arm64.sh
```

Or manually:

```bash
docker buildx build \
  --platform linux/arm64 \
  -f host/dockerfile \
  -t 192.168.178.103:5000/ai-platform-host:latest \
  --push .
```

---

# 2️⃣ Sync K3s Manifests

```bash
./scripts/sync_k3s_manifests.sh
```

Ensure manifests exist on Raspberry:

```bash
~/workdir/k3s/manifests/ai_platform/
```

---

# 3️⃣ Deploy AI Host

On Raspberry:

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/ai-platform-host.yml

kubectl rollout restart deployment/ai-platform-host -n ai-platform

kubectl rollout status deployment/ai-platform-host -n ai-platform
```

---

# 4️⃣ Verify Host

```bash
kubectl get pods -n ai-platform -o wide
kubectl get svc -n ai-platform
kubectl logs -n ai-platform deployment/ai-platform-host
```

From laptop:

```bash
curl http://<node-ip>:30080/health
```

Example:

```bash
curl http://100.109.72.92:30080/health
```

---

# 5️⃣ Deploy Ollama

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/ollama.yml
```

Verify:

```bash
kubectl get pods -n ai-platform -o wide
kubectl logs -n ai-platform deployment/ollama
```

---

# 6️⃣ Verify Ollama API

Run test pod:

```bash
kubectl run curl-test \
  -n ai-platform \
  --rm -it \
  --image=curlimages/curl \
  --restart=Never \
  -- http://ollama:11434/api/tags
```

---

# 7️⃣ Manage Models in Ollama

List models:

```bash
kubectl exec -n ai-platform deployment/ollama -- ollama list
```

Pull models:

```bash
kubectl exec -n ai-platform deployment/ollama -- ollama pull llama3
kubectl exec -n ai-platform deployment/ollama -- ollama pull llama3.2:1b
kubectl exec -n ai-platform deployment/ollama -- ollama pull nomic-embed-text
```

Remove models:

```bash
kubectl exec -n ai-platform deployment/ollama -- ollama rm llama3
```

---

# 8️⃣ Test Chat API (LLM Only)

From laptop:

```bash
curl -X POST http://<node-ip>:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"What is the weather in Berlin?"}'
```

---

# 9️⃣ Add RAG Artifacts (Manual Sync)

Create directory on Raspberry:

```bash
mkdir -p /home/roman/nfs/rag/artifacts
```

Sync from laptop:

```bash
rsync -avz artifacts/rag/ roman@raspberry:/home/roman/nfs/rag/artifacts/
```

Verify:

```bash
ls -la /home/roman/nfs/rag/artifacts
```

---

# 🔟 Restart Host with RAG

```bash
kubectl rollout restart deployment/ai-platform-host -n ai-platform
kubectl logs -n ai-platform deployment/ai-platform-host
```

---

# 1️⃣1️⃣ Test Retrieval Flow

```bash
curl -X POST http://<node-ip>:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

---

# 🧠 Query Modes

## Tool-first

```bash
"What is the weather in Berlin?"
```

## Retrieval-first

```bash
"How do I request vacation?"
```

---

# 🔥 Troubleshooting

## Host returns 500

Check logs:

```bash
kubectl logs -n ai-platform deployment/ai-platform-host
```

Common causes:

* Ollama not reachable
* wrong model name
* embedding model missing

---

## Ollama empty model list

Fix:

```bash
kubectl exec -n ai-platform deployment/ollama -- ollama pull <model>
```

---

## No RAG results

Check:

* artifacts path
* NFS mount inside pod
* embedding model installed

---

# 📌 Summary

You now have:

* AI host running in K3s ✔
* Ollama as LLM backend ✔
* chat API working ✔
* RAG integration via NFS ✔

---

# 🚀 Next Step

👉 Automate RAG pipeline using Slurm + Apptainer
