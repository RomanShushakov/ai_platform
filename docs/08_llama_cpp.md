# ⚡ llama.cpp Runtime on Jetson

This step builds and runs **llama.cpp** as the main GPU-backed LLM runtime on Jetson.

llama.cpp is used because it gives direct control over:

* GGUF model files
* GPU offload
* runtime flags
* OpenAI-compatible API
* future LoRA adapters

---

# 🎯 Goal

After this step, you will have:

* llama.cpp built with CUDA on Jetson
* a Docker runtime image for `llama-server`
* GGUF models stored on NFS
* chat inference working
* embeddings inference working (optional)
* baseline performance verified

---

# 🧭 Architecture

```text
Jetson
  ↓
llama.cpp build
  ↓
llama-server binary
  ↓
Docker runtime image
  ↓
GGUF models from NFS
  ↓
OpenAI-compatible APIs
  (chat + embeddings)
```

---

# 1️⃣ Install Build Dependencies

### on jetson:

```bash
sudo apt update
sudo apt install -y git cmake build-essential curl
```

Check CUDA:

```bash
which nvcc || true
ls -l /usr/local/cuda/bin/nvcc || true
```

If needed:

```bash
sudo apt install -y cuda-nvcc-12-6
```

---

# 2️⃣ Build llama.cpp with CUDA

### on jetson:

```bash
cd ~/workdir

git clone https://github.com/ggml-org/llama.cpp.git
cd llama.cpp
```

Configure:

```bash
cmake -B build \
  -DGGML_CUDA=ON \
  -DCMAKE_CUDA_COMPILER=/usr/local/cuda/bin/nvcc \
  -DCMAKE_BUILD_TYPE=Release
```

Build:

```bash
cmake --build build --config Release -j$(nproc)
```

Verify:

```bash
~/workdir/llama.cpp/build/bin/llama-server --help | head -40
```

Expected:

```text
ggml_cuda_init: found 1 CUDA devices
Device 0: Orin
```

---

# 3️⃣ Sync Runtime Image Files

### on laptop:

```bash
./scripts/sync_llama_cpp_runtime_image_to_jetson.sh
```

This syncs:

```text
infra/images/llama_cpp_runtime/
```

to:

```text
/home/roman/workdir/ai_platform/infra/images/llama_cpp_runtime/
```

---

# 4️⃣ Copy Runtime Binary and Libraries

### on jetson:

```bash
cp ~/workdir/llama.cpp/build/bin/llama-server \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/llama-server
```

Inspect dependencies:

```bash
ldd ~/workdir/llama.cpp/build/bin/llama-server
```

Copy libs:

```bash
cp ~/workdir/llama.cpp/build/bin/libllama*.so* \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/

cp ~/workdir/llama.cpp/build/bin/libggml*.so* \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/

cp ~/workdir/llama.cpp/build/bin/libmtmd*.so* \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/
```

---

# 5️⃣ Build Runtime Docker Image

### on jetson:

```bash
cd ~/workdir/ai_platform/infra/images/llama_cpp_runtime

docker build \
  -t 192.168.178.103:5000/llama-cpp-server:latest \
  .
```

If needed, configure insecure registry:

```bash
sudo vi /etc/docker/daemon.json
```

```json
{
  "insecure-registries": ["192.168.178.103:5000"]
}
```

Restart Docker:

```bash
sudo systemctl restart docker
```

Push:

```bash
docker push 192.168.178.103:5000/llama-cpp-server:latest
```

---

# 6️⃣ Download GGUF Models

### on jetson:

```bash
mkdir -p /home/roman/nfs/models/gguf
cd /home/roman/nfs/models/gguf
```

Chat model:

```bash
wget -O qwen2.5-0.5b-instruct-q4_k_m.gguf \
  https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

Embedding model:

```bash
wget -O all-minilm-l6-v2-q8_0.gguf \
  https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-q8_0.gguf
```

---

# 7️⃣ Run llama.cpp (Chat)

### on jetson:

```bash
docker run --rm -it \
  --runtime nvidia \
  --network host \
  -v /home/roman/nfs/models/gguf:/models \
  192.168.178.103:5000/llama-cpp-server:latest \
  -m /models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 0.0.0.0 \
  --port 8000 \
  -ngl 99 \
  -c 1024 \
  --parallel 1
```

---

# 8️⃣ Run llama.cpp (Embeddings)

### on jetson:

```bash
docker run --rm -it \
  --runtime nvidia \
  --network host \
  -v /home/roman/nfs/models/gguf:/models \
  192.168.178.103:5000/llama-cpp-server:latest \
  -m /models/all-minilm-l6-v2-q8_0.gguf \
  --host 0.0.0.0 \
  --port 8001 \
  --embeddings \
  --pooling mean \
  -ngl 99 \
  -c 512 \
  --parallel 1
```

---

# 9️⃣ Test APIs

### on raspberry:

Chat:

```bash
curl http://jetson:8000/v1/models
```

```bash
curl -X POST http://jetson:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages": [
      {"role": "user", "content": "Say hello"}
    ]
  }'
```

Embeddings:

```bash
curl -X POST http://jetson:8001/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{
    "model": "all-minilm-l6-v2-q8_0.gguf",
    "input": "How do I request vacation?"
  }'
```

---

# 🔟 Enable Jetson Performance Mode

### on jetson:

```bash
sudo jetson_clocks
```

---

# 🧪 Performance Notes

* cold start is slow
* warm cache improves latency significantly
* small models can reach ~100 tokens/sec

---

# 🔥 Troubleshooting

## Missing shared libraries

Re-copy libs and rebuild image.

## Docker push fails

Fix insecure registry config.

## Slow inference

```bash
sudo jetson_clocks
```

---

# 📌 Summary

You now have:

* llama.cpp chat ✔
* llama.cpp embeddings ✔
* Docker runtime ✔
* GPU inference ✔

---

# 🚀 Next Step

👉 Deploy llama.cpp into K3s and integrate with the platform
