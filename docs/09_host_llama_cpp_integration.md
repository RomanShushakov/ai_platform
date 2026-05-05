# 🧠 llama.cpp Integration into AI Platform

This step integrates llama.cpp into the platform and replaces Ollama.

---

# 🎯 Goal

After this step:

* chat → llama.cpp
* embeddings → llama.cpp
* RAG rebuilt using new embeddings
* Ollama removed

---

# 🧭 Architecture

```text
User
  ↓
K3s Service
  ↓
Rust Host
  ├── llama.cpp (chat)
  ├── llama.cpp (embeddings)
  └── Retriever (NFS artifacts)
```

---

# 1️⃣ Deploy Chat Runtime

### on laptop:

```bash
./scripts/sync_k3s_manifests.sh
```

### on raspberry:

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/llama-cpp.yml

kubectl get pods -n ai-platform -o wide
kubectl logs -n ai-platform deployment/llama-cpp
```

Test:

```bash
kubectl run llama-test \
  -n ai-platform \
  --rm -it \
  --image=curlimages/curl \
  --restart=Never \
  -- curl http://llama-cpp:8000/v1/models
```

---

# 2️⃣ Deploy Warmup Job

### on raspberry:

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/llama-cpp-warmup.yml

kubectl logs -n ai-platform job/llama-cpp-warmup
```

---

# 3️⃣ Deploy Embeddings Runtime

### on laptop:

```bash
./scripts/sync_k3s_manifests.sh
```

### on raspberry:

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/llama-cpp-embed.yml

kubectl rollout status deployment/llama-cpp-embed -n ai-platform
kubectl logs -n ai-platform deployment/llama-cpp-embed
```

Test:

```bash
kubectl run embed-test \
  -n ai-platform \
  --rm -it \
  --image=curlimages/curl \
  --restart=Never \
  -- \
  curl -X POST http://llama-cpp-embed:8001/v1/embeddings \
    -H "Content-Type: application/json" \
    -d '{"model":"all-minilm-l6-v2-q8_0.gguf","input":"test"}'
```

---

# 4️⃣ Update Embedding Client

Switch:

```text
/api/embeddings → /v1/embeddings
```

Parse:

```text
data[0].embedding
```

---

# 5️⃣ Rebuild RAG Artifacts

### on laptop:

```bash
./scripts/build_push_rag_indexer_arm64.sh
```

### on jetson:

```bash
rm -f /home/roman/nfs/rag/images/rag-indexer.sif

apptainer pull \
  --no-https \
  /home/roman/nfs/rag/images/rag-indexer.sif \
  docker://192.168.178.103:5000/rag-indexer:latest
```

### on raspberry:

```bash
sbatch /home/roman/nfs/rag/jobs/rag-index.slurm

squeue
ls -lh /home/roman/nfs/rag/jobs/logs
tail -n 100 /home/roman/nfs/rag/jobs/logs/*.out
```

---

# 6️⃣ Restart Host

### on laptop:

```bash
./scripts/build_push_ai_platform_host_arm64.sh
./scripts/sync_k3s_manifests.sh
```

### on raspberry:

```bash
kubectl apply -f ~/workdir/k3s/manifests/ai_platform/ai-platform-host.yml

kubectl rollout restart deployment/ai-platform-host -n ai-platform
kubectl rollout status deployment/ai-platform-host -n ai-platform
kubectl logs -n ai-platform deployment/ai-platform-host
```

---

# 7️⃣ Verify End-to-End

### on laptop:

```bash
curl -X POST http://jetson.tail0140c.ts.net:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

Expected:

* retrieval used
* correct answer
* sources returned

---

# 8️⃣ Remove Ollama

### on raspberry:

```bash
kubectl delete -f ~/workdir/k3s/manifests/ai_platform/ollama.yml

kubectl get pods -n ai-platform -o wide
```

---

# 🧠 Key Learnings

* llama.cpp API differs from Ollama
* embeddings endpoint differs
* strict JSON parsing required
* warmup improves latency significantly

---

# 📌 Final State

* llama.cpp chat ✔
* llama.cpp embeddings ✔
* RAG rebuilt ✔
* Slurm pipeline working ✔
* Ollama removed ✔

---

# 🚀 Next Step

👉 Implement LoRA support
