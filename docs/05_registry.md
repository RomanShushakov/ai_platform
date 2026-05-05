# 🐳 Private Docker Registry (K3s / Jetson)

This step sets up and operates a **local Docker registry** running on the Jetson.

It is used to:

* push ARM64 images built on laptop
* serve images to K3s cluster
* avoid Docker Hub dependency
* enable fast local iteration

---

# 🎯 Goal

After this step, you can:

* build ARM64 images locally
* push them to Jetson registry
* deploy them in K3s

---

# 🧭 Registry Endpoint

```text
http://192.168.178.103:5000
```

---

# 1️⃣ Verify Registry

List repositories:

```bash
curl http://192.168.178.103:5000/v2/_catalog
```

---

Check specific image manifest:

```bash
curl -I \
  -H "Accept: application/vnd.docker.distribution.manifest.v2+json" \
  http://192.168.178.103:5000/v2/nginx-test/manifests/latest
```

---

# 2️⃣ Delete Image (Manual Cleanup)

⚠️ Registry API requires **digest**, not tag.

Delete image:

```bash
curl -X DELETE \
  http://192.168.178.103:5000/v2/nginx-test/manifests/<digest>
```

Example:

```bash
curl -X DELETE \
  http://192.168.178.103:5000/v2/nginx-test/manifests/sha256:...
```

---

# 3️⃣ Garbage Collection (Jetson)

Run on Jetson:

```bash
sudo docker-registry garbage-collect /etc/docker/registry/config.yml
```

If cleanup fails or repo is stuck:

```bash
sudo systemctl stop docker-registry

sudo rm -rf /var/lib/docker-registry/docker/registry/v2/repositories/nginx-test

sudo docker-registry garbage-collect /etc/docker/registry/config.yml

sudo systemctl start docker-registry
```

---

# 4️⃣ Enable Insecure Registry (Jetson)

Edit:

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

---

# 5️⃣ Build ARM64 Image (from Laptop)

Use Docker Buildx:

```bash
docker buildx build \
  --platform linux/arm64 \
  -f host/dockerfile \
  -t 192.168.178.103:5000/ai-platform-host:latest \
  --push .
```

---

# 6️⃣ Push Test Image

Example:

```bash
docker pull nginx:alpine

docker tag nginx:alpine 192.168.178.103:5000/nginx-test:latest

docker push 192.168.178.103:5000/nginx-test:latest
```

---

# 7️⃣ Use Image in K3s

Deploy from Raspberry:

```bash
kubectl create deployment registry-test \
  --image=192.168.178.103:5000/nginx-test:latest

kubectl expose deployment registry-test \
  --type=NodePort \
  --port=80
```

Verify:

```bash
kubectl get pods -o wide
kubectl get svc registry-test
```

---

# 🔥 Troubleshooting

## Image stuck on "Pulling"

Check:

* registry reachable from node
* correct IP (Jetson LAN IP!)
* insecure registry configured

---

## Wrong architecture

If pod crashes immediately:

* image built without `--platform linux/arm64`

---

## Registry full / corrupted

Fix:

```bash
stop registry → delete repo → garbage collect → start registry
```

---

# 📌 Summary

You now have:

* private ARM64 registry ✔
* local image distribution ✔
* fast dev → deploy loop ✔

---

# 🚀 Next Step

👉 Deploy AI platform services into K3s:

* host
* Ollama
* later: llama.cpp
