#!/usr/bin/env bash
#SBATCH --job-name=rag-index
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --nodelist=jetson
#SBATCH --output=/home/roman/nfs/rag/jobs/logs/rag-index-%j.out
#SBATCH --error=/home/roman/nfs/rag/jobs/logs/rag-index-%j.err

set -euo pipefail

export RAG_ROOT=/home/roman/nfs/rag
export RAG_IMAGE="${RAG_ROOT}/images/rag-indexer.sif"
export KNOWLEDGE_BASE_PATH="${RAG_ROOT}/knowledge_base"
export RAG_OUTPUT_DIR="${RAG_ROOT}/artifacts"
export RAG_WORK_DIR="${RAG_ROOT}/jobs/work/${SLURM_JOB_ID}"
export OLLAMA_BASE_URL="http://127.0.0.1:31134"
export EMBEDDING_MODEL="nomic-embed-text"

echo "RAG index job started"
echo "Job ID: ${SLURM_JOB_ID}"
echo "Node: $(hostname)"
echo "Image: ${RAG_IMAGE}"
echo "Knowledge base: ${KNOWLEDGE_BASE_PATH}"
echo "Output dir: ${RAG_OUTPUT_DIR}"
echo "Work dir: ${RAG_WORK_DIR}"
echo "Ollama: ${OLLAMA_BASE_URL}"
echo "Embedding model: ${EMBEDDING_MODEL}"

mkdir -p "${RAG_WORK_DIR}"
mkdir -p "${RAG_OUTPUT_DIR}"

if [ ! -f "${RAG_IMAGE}" ]; then
  echo "ERROR: Apptainer image missing: ${RAG_IMAGE}" >&2
  exit 1
fi

if [ ! -d "${KNOWLEDGE_BASE_PATH}" ]; then
  echo "ERROR: knowledge base directory missing: ${KNOWLEDGE_BASE_PATH}" >&2
  exit 1
fi

apptainer exec \
  --bind "${RAG_ROOT}:${RAG_ROOT}" \
  "${RAG_IMAGE}" \
  /usr/local/bin/rag_indexer

echo "Generated artifacts:"
ls -lh "${RAG_OUTPUT_DIR}"

echo "RAG index job finished successfully"
