# 🧠 RAG Rebuild Pipeline with Slurm + Apptainer

This step turns RAG indexing from a manual local process into a **batch pipeline** executed through Slurm.

The pipeline runs on Jetson and writes updated artifacts to NFS.

---

# 🎯 Goal

After this step, you will have:

* RAG knowledge base synced to NFS
* `rag_indexer` packaged as an ARM64 container image
* Apptainer `.sif` image created on Jetson
* Slurm job rebuilding RAG artifacts
* K3s host consuming regenerated artifacts

---

# 🧭 Architecture

```text
Laptop
  ↓ sync knowledge_base / build image
Raspberry
  ↓ sbatch
Slurm
  ↓ schedules job
Jetson
  ↓ Apptainer runs rag_indexer
NFS
  ↓ updated artifacts
K3s Host
  ↓ reads artifacts
/chat
```

---

# 1️⃣ Sync Knowledge Base to NFS

From laptop:

```bash
./scripts/sync_rag_knowledge_base.sh
```

Expected target:

```text
/home/roman/nfs/rag/knowledge_base
```

---

# 2️⃣ Build and Push RAG Indexer Image

From laptop:

```bash
./scripts/build_push_rag_indexer_arm64.sh
```

This builds the `rag_indexer` crate for `linux/arm64` and pushes it to the local registry:

```text
192.168.178.103:5000/rag-indexer:latest
```

Verify:

```bash
curl http://192.168.178.103:5000/v2/_catalog
```

Expected:

```text
rag-indexer
```

---

# 3️⃣ Create Apptainer Image on Jetson

On Jetson:

```bash
mkdir -p /home/roman/nfs/rag/images
```

Pull from local HTTP registry:

```bash
apptainer pull \
  --no-https \
  /home/roman/nfs/rag/images/rag-indexer.sif \
  docker://192.168.178.103:5000/rag-indexer:latest
```

Expected output:

```text
/home/roman/nfs/rag/images/rag-indexer.sif
```

---

# 4️⃣ Test RAG Indexer Manually via Apptainer

On Jetson:

```bash
KNOWLEDGE_BASE_PATH=/home/roman/nfs/rag/knowledge_base \
RAG_OUTPUT_DIR=/home/roman/nfs/rag/artifacts \
EMBEDDING_BASE_URL=http://127.0.0.1:31134 \
EMBEDDING_MODEL=nomic-embed-text \
apptainer exec \
  --bind /home/roman/nfs/rag:/home/roman/nfs/rag \
  /home/roman/nfs/rag/images/rag-indexer.sif \
  /usr/local/bin/rag_indexer
```

Expected:

```text
Loading knowledge base from /home/roman/nfs/rag/knowledge_base
Loaded N chunks
Embedding ...
Indexing complete → /home/roman/nfs/rag/artifacts
```

Artifacts:

```text
/home/roman/nfs/rag/artifacts/
  chunks.json
  embeddings.json
  manifest.json
```

---

# 5️⃣ Prepare Slurm Job Directories

On Raspberry:

```bash
mkdir -p /home/roman/nfs/rag/jobs/logs
mkdir -p /home/roman/nfs/rag/jobs/work
```

---

# 6️⃣ RAG Index Slurm Job

Job file:

```text
/home/roman/nfs/rag/jobs/rag-index.slurm
```

The job should run:

```bash
apptainer exec \
  --bind /home/roman/nfs/rag:/home/roman/nfs/rag \
  /home/roman/nfs/rag/images/rag-indexer.sif \
  /usr/local/bin/rag_indexer
```

Important env vars:

```bash
export RAG_ROOT=/home/roman/nfs/rag
export RAG_IMAGE="${RAG_ROOT}/images/rag-indexer.sif"
export KNOWLEDGE_BASE_PATH="${RAG_ROOT}/knowledge_base"
export RAG_OUTPUT_DIR="${RAG_ROOT}/artifacts"
export RAG_WORK_DIR="${RAG_ROOT}/jobs/work/${SLURM_JOB_ID}"
export EMBEDDING_BASE_URL="http://127.0.0.1:31134"
export EMBEDDING_MODEL="nomic-embed-text"
```

---

# 7️⃣ Run RAG Rebuild via Slurm

On Raspberry:

```bash
sbatch /home/roman/nfs/rag/jobs/rag-index.slurm
```

Check queue:

```bash
squeue
```

Check logs:

```bash
ls -lh /home/roman/nfs/rag/jobs/logs

tail -n 100 /home/roman/nfs/rag/jobs/logs/rag-index-*.out
tail -n 100 /home/roman/nfs/rag/jobs/logs/rag-index-*.err
```

Expected:

```text
RAG index job started
Node: jetson
Image: /home/roman/nfs/rag/images/rag-indexer.sif
Loaded N chunks
Embedding ...
Generated artifacts:
chunks.json
embeddings.json
manifest.json
RAG index job finished successfully
```

---

# 8️⃣ Restart Host to Reload Artifacts

On Raspberry:

```bash
kubectl rollout restart deployment/ai-platform-host -n ai-platform
kubectl rollout status deployment/ai-platform-host -n ai-platform
kubectl logs -n ai-platform deployment/ai-platform-host
```

---

# 9️⃣ Verify RAG Response

From laptop:

```bash
curl -X POST http://100.90.183.16:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

Or via MagicDNS:

```bash
curl -X POST http://jetson.tail0140c.ts.net:30080/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"How do I request vacation?"}'
```

Expected:

```text
Retrieved 1 chunk(s)
source: Vacation Policy
```

---

# 🔥 Troubleshooting

## Apptainer registry pull fails with HTTPS error

Symptom:

```text
http: server gave HTTP response to HTTPS client
```

Fix:

```bash
apptainer pull --no-https ...
```

---

## Slurm job fails with missing GLIBC

Cause:

* binary was cross-compiled against newer glibc

Fix:

* run via Apptainer image
* or build natively on Jetson

---

## RAG artifacts missing

Check:

```bash
ls -lh /home/roman/nfs/rag/artifacts
```

Expected:

```text
chunks.json
embeddings.json
manifest.json
```

---

## Ollama not reachable from Slurm job

The RAG job uses:

```text
http://127.0.0.1:31134
```

This assumes Ollama is exposed through NodePort / host-accessible port on Jetson.

Verify on Jetson:

```bash
curl http://127.0.0.1:31134/api/tags
```

---

# 📌 Summary

You now have a reproducible RAG batch pipeline:

* source docs synced to NFS ✔
* indexer packaged as container ✔
* Apptainer runs workload ✔
* Slurm schedules job ✔
* artifacts regenerated on NFS ✔
* K3s host consumes updated RAG data ✔

---

# 🚀 Next Step

👉 Deploy and tune llama.cpp as the production-like LLM runtime.
