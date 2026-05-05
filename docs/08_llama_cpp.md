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
* a GGUF model stored on NFS
* manual inference working
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
GGUF model from NFS
  ↓
OpenAI-compatible API
```

---

# 1️⃣ Install Build Dependencies

On Jetson:

```bash
sudo apt update
sudo apt install -y git cmake build-essential curl
```

If CUDA compiler is missing:

```bash
which nvcc || true
ls -l /usr/local/cuda/bin/nvcc || true
```

If needed, install CUDA compiler package:

```bash
sudo apt install -y cuda-nvcc-12-6
```

---

# 2️⃣ Build llama.cpp with CUDA

On Jetson:

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

From laptop:

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

On Jetson:

```bash
cp ~/workdir/llama.cpp/build/bin/llama-server \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/llama-server
```

Inspect dependencies:

```bash
ldd ~/workdir/llama.cpp/build/bin/llama-server
```

Copy llama.cpp shared libraries:

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

On Jetson:

```bash
cd ~/workdir/ai_platform/infra/images/llama_cpp_runtime
```

Build:

```bash
docker build \
  -t 192.168.178.103:5000/llama-cpp-server:latest \
  .
```

If Docker cannot push to the HTTP registry, configure insecure registry:

```bash
sudo vi /etc/docker/daemon.json
```

Add:

```json
{
  "insecure-registries": ["192.168.178.103:5000"]
}
```

Restart Docker:

```bash
sudo systemctl restart docker
```

Verify:

```bash
docker info | grep -i insecure -A2
```

Push:

```bash
docker push 192.168.178.103:5000/llama-cpp-server:latest
```

---

# 6️⃣ Download GGUF Model

On Jetson:

```bash
mkdir -p /home/roman/nfs/models/gguf

cd /home/roman/nfs/models/gguf
```

Download Qwen2.5 0.5B Instruct:

```bash
wget -O qwen2.5-0.5b-instruct-q4_k_m.gguf \
  https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

Expected:

```text
/home/roman/nfs/models/gguf/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

---

# 7️⃣ Run llama.cpp in Docker

On Jetson:

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

# 8️⃣ Run llama.cpp Natively

Useful for comparison/debugging:

```bash
~/workdir/llama.cpp/build/bin/llama-server \
  -m /home/roman/nfs/models/gguf/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 0.0.0.0 \
  --port 8000 \
  -ngl 99 \
  -c 1024 \
  --parallel 1
```

---

# 9️⃣ Test OpenAI-Compatible API

List models:

```bash
curl http://jetson:8000/v1/models
```

Chat completion:

```bash
curl -X POST http://jetson:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5-0.5b-instruct-q4_k_m.gguf",
    "messages": [
      {"role": "user", "content": "Say hello in one short sentence."}
    ],
    "temperature": 0,
    "max_tokens": 32
  }'
```

Expected:

```json
{
  "choices": [
    {
      "message": {
        "content": "Hello! ..."
      }
    }
  ]
}
```

---

# 🔟 Enable Jetson Performance Mode

Check current mode:

```bash
sudo nvpmodel -q
sudo jetson_clocks --show
```

Enable max clocks:

```bash
sudo jetson_clocks
```

Verify:

```bash
sudo jetson_clocks --show
```

Expected:

```text
GPU CurrentFreq close to max
EMC CurrentFreq close to max
```

This significantly improves inference performance.

---

# 🧪 Performance Notes

Cold requests may be slow.

After warmup / prompt cache:

```text
generation can reach ~100 tokens/sec on tiny models
```

Observed useful runtime flags:

```bash
-ngl 99
-c 1024
--parallel 1
```

Avoid over-tuning too early. Some flags reduced performance:

```bash
-b 128
-ub 32
-fa off
```

---

# 🔥 Troubleshooting

## Missing shared library

Example:

```text
error while loading shared libraries: libllama-common.so.0
```

Fix:

```bash
cp ~/workdir/llama.cpp/build/bin/libllama*.so* \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/

cp ~/workdir/llama.cpp/build/bin/libggml*.so* \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/

cp ~/workdir/llama.cpp/build/bin/libmtmd*.so* \
  ~/workdir/ai_platform/infra/images/llama_cpp_runtime/
```

Then rebuild the image.

---

## Docker push fails with HTTPS error

Symptom:

```text
http: server gave HTTP response to HTTPS client
```

Fix Docker insecure registry config:

```json
{
  "insecure-registries": ["192.168.178.103:5000"]
}
```

Then:

```bash
sudo systemctl restart docker
```

---

## Inference very slow

Check clocks:

```bash
sudo jetson_clocks --show
```

Fix:

```bash
sudo jetson_clocks
```

---

## Runtime image is large

Current image may be large because it uses Jetson runtime base and CUDA dependencies.

This is acceptable for now.

Future optimization:

* smaller L4T base image
* copy only required runtime libs
* avoid full JetPack base

---

# 📌 Summary

You now have:

* llama.cpp built with CUDA ✔
* GGUF model stored on NFS ✔
* Docker runtime image ✔
* OpenAI-compatible API ✔
* GPU-backed inference on Jetson ✔

---

# 🚀 Next Step

👉 Deploy llama.cpp into K3s and connect the Rust host to it.
