# 🖥 Infra Track: Slurm + Shared Storage + Future K3s

This directory tracks the infrastructure side of the AI platform project.

The goal is not just to run a chatbot, but to understand how:

- online serving
- batch compute
- shared storage
- deployment platforms

fit together in a small real lab.

---

# 🎯 Current Infra Direction

We use a split architecture:

- **Rust AI platform** = online serving/orchestration plane
- **Slurm cluster** = offline/batch compute plane
- **shared storage (NFS)** = bridge between them
- **future K3s** = always-on application deployment plane

This separation keeps the online app responsive while delegating heavy work to scheduled batch jobs.

---

# 🧩 Roles of Each Layer

## 1. Rust AI Platform

Handles:

- chat/API requests
- orchestration and tools
- RAG query-time retrieval
- model backend integration

Runs:

- locally
- Docker Compose
- later in K3s

---

## 2. Slurm (Batch Plane)

Handles:

- RAG rebuilds
- embeddings generation
- LoRA / QLoRA training
- evaluation jobs
- preprocessing

Slurm = **offline compute engine**

---

## 3. Shared Storage (Jetson-based NFS)

### Setup

- Server: Jetson
- Client: Raspberry
- Shared path:

```
/home/roman/nfs
```

### Why Jetson as NFS server

- larger disk (256GB)
- jobs execute locally on Jetson
- avoids network write overhead
- realistic infra design

---

# 📦 Storage Layout

```
/home/roman/nfs/
├── slurm/
│   └── job-examples/
├── rag/
├── models/
└── logs/
```

Used for:

- job scripts
- outputs
- artifacts
- checkpoints

---

## 4. Future K3s

K3s will host:

- Rust API
- tools server
- UI (later)

It will mount NFS to access:

- RAG artifacts
- model outputs
- logs

---

# 🧠 Retrieval Strategy

Current:

- JSON artifacts
  - chunks.json
  - embeddings.json
  - manifest.json

Why:

- simple
- debuggable
- no extra infra

Future (optional):

- Qdrant if needed

---

# 🖥 Hardware Topology

Laptop:
- dev
- ansible
- client

Raspberry:
- Slurm controller
- NFS client
- future K3s control plane

Jetson:
- Slurm worker
- GPU node
- NFS server
- future K3s worker

---

# ✅ Current Working Milestone

Fully working:

- Slurm cluster
- GPU scheduling (GRES)
- NFS shared storage
- distributed job execution

---

## Verified Behavior

- jobs submitted from Raspberry
- executed on Jetson
- outputs written to NFS
- results visible instantly from Raspberry

Example:

```
hostname → jetson
output → /home/roman/nfs/slurm/job-examples/slurm-XX.out
```

---

# 🧪 Example Jobs

## Local (non-NFS)
```
~/workdir/slurm/job-examples
```

## Shared (NFS)
```
~/nfs/slurm/job-examples
```

Includes:

- hostname.sbatch
- sleep.sbatch
- python_version.sbatch
- gpu_probe.sbatch
- gpu_probe_gres.sbatch

---

# 🧭 Architecture Flow

## Online

User → Rust app → tools/RAG/model → response

## Offline

Trigger → Slurm → NFS → app consumes

---

# 🛠 Batch Model

## RAG rebuild

- Slurm job processes docs
- writes artifacts to `/nfs/rag`
- app reads them

## Training

- Slurm job runs training
- writes checkpoints/logs to `/nfs/models`

---

# 📦 Container Strategy

- app → Docker / K3s
- batch → Slurm (+ future Apptainer)

---

# 🚀 Next Roadmap

1. Slurm + NFS ✅
2. K3s cluster setup
3. Deploy Rust app
4. Mount NFS into pods
5. Connect batch → app
6. Optional: Qdrant

---

# 🧠 Design Principles

- keep it simple
- separate online/offline
- shared storage as bridge
- reproducible via Ansible
- avoid overengineering

---

# 📌 Summary

You now have:

- working Slurm cluster
- GPU scheduling
- shared storage (NFS)
- real batch pipeline

Next step:

👉 **K3s (deployment layer)**
