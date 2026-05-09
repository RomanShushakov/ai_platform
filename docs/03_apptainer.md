<!--
AI Platform Lab Documentation
Standardized Edition
Environment:
- Laptop: development machine
- Raspberry Pi: Slurm controller + K3s control-plane
- Jetson Orin Nano: GPU worker + inference + training node
- External Tailscale endpoint: 100.109.72.92
-->

# 📦 Apptainer Setup (HPC Containers)

This step installs **Apptainer (formerly Singularity)** on your nodes.

Apptainer is used for:

* running containerized workloads inside **Slurm**
* avoiding Docker dependency on compute nodes
* reproducible HPC jobs

---

# 🎯 Goal

After this step, you will be able to:

* run containers inside Slurm jobs
* package workloads as `.sif` images
* use GPU-enabled containers in batch jobs

---

# 🧭 When to Use Apptainer

Use Apptainer when:

* running jobs via Slurm
* executing batch pipelines (RAG, training, preprocessing)
* needing reproducibility across nodes

Do **not** use it for:

* K3s workloads (use Docker/OCI instead)
* long-running services

---

# 1️⃣ Install Apptainer

Run:

```bash
ansible-playbook apptainer/playbooks/install_apptainer.yml -K
```

This will:

* install Apptainer on all nodes
* configure required dependencies
* enable user-level container execution

---

# 2️⃣ Verify Installation

On any node:

```bash
apptainer --version
```

Optional quick test:

```bash
apptainer exec docker://alpine echo "hello from apptainer"
```

Expected output:

```text
hello from apptainer
```

---

# 🔄 Rollback (if needed)

```bash
ansible-playbook apptainer/playbooks/install_apptainer_rollback.yml -K
```

---

# 🧠 How It Fits the System

```text
Slurm Job
   ↓
Apptainer Container (.sif)
   ↓
Runs on Jetson (GPU)
   ↓
Writes results to NFS
```

---

# 🔥 Example Future Usage

```bash
apptainer exec my-job.sif python run_indexer.py
```

or inside Slurm:

```bash
srun apptainer exec my-job.sif python job.py
```

---

# 📌 Summary

You now have:

* HPC-friendly container runtime ✔
* Slurm-compatible execution ✔
* foundation for reproducible batch pipelines ✔

---

# 🚀 Next Step

👉 Continue with K3s setup:

* cluster deployment
* service orchestration
* AI workloads
