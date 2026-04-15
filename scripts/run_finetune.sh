#!/usr/bin/env bash
set -euo pipefail

MODEL_NAME="${MODEL_NAME:-Qwen/Qwen2.5-1.5B-Instruct}"
TRAIN_FILE="${TRAIN_FILE:-data/curated/research_agent_train.jsonl}"
RUN_NAME="${RUN_NAME:-$(date +%Y%m%d_%H%M%S)}"
BATCH_SIZE="${BATCH_SIZE:-1}"
GRAD_ACCUM="${GRAD_ACCUM:-16}"
CONTEXT_LENGTH="${CONTEXT_LENGTH:-2048}"
SAVE_STEPS="${SAVE_STEPS:-200}"
MAX_STEPS="${MAX_STEPS:-1000}"
LEARNING_RATE="${LEARNING_RATE:-2e-4}"

BASE_DIR="models/finetuned"
ADAPTER_DIR="$BASE_DIR/adapters/$RUN_NAME"
MERGED_DIR="$BASE_DIR/merged/$RUN_NAME"
GGUF_DIR="$BASE_DIR/gguf/$RUN_NAME"

mkdir -p "$ADAPTER_DIR" "$MERGED_DIR" "$GGUF_DIR"

echo "[1/3] Training QLoRA adapters -> $ADAPTER_DIR"
python scripts/train_qlora.py \
  --model "$MODEL_NAME" \
  --train-file "$TRAIN_FILE" \
  --output-adapter-dir "$ADAPTER_DIR" \
  --batch-size "$BATCH_SIZE" \
  --grad-accum "$GRAD_ACCUM" \
  --context-length "$CONTEXT_LENGTH" \
  --save-steps "$SAVE_STEPS" \
  --max-steps "$MAX_STEPS" \
  --learning-rate "$LEARNING_RATE"

echo "[2/3] Merging adapters -> $MERGED_DIR"
ADAPTER_DIR="$ADAPTER_DIR" MERGED_DIR="$MERGED_DIR" python - <<'PY'
import os
from pathlib import Path
from peft import AutoPeftModelForCausalLM
from transformers import AutoTokenizer

adapter_dir = Path(os.environ["ADAPTER_DIR"])
merged_dir = Path(os.environ["MERGED_DIR"])
merged_dir.mkdir(parents=True, exist_ok=True)

model = AutoPeftModelForCausalLM.from_pretrained(adapter_dir, device_map="auto")
merged = model.merge_and_unload()
merged.save_pretrained(merged_dir)

tokenizer = AutoTokenizer.from_pretrained(adapter_dir, use_fast=True)
tokenizer.save_pretrained(merged_dir)
PY

echo "[3/3] Optional GGUF export -> $GGUF_DIR"
if command -v convert_hf_to_gguf.py >/dev/null 2>&1; then
  convert_hf_to_gguf.py "$MERGED_DIR" --outfile "$GGUF_DIR/model-q4_k_m.gguf" --outtype q4_k_m
  convert_hf_to_gguf.py "$MERGED_DIR" --outfile "$GGUF_DIR/model-q5_k_m.gguf" --outtype q5_k_m
elif [[ -f llama.cpp/convert_hf_to_gguf.py ]]; then
  python llama.cpp/convert_hf_to_gguf.py "$MERGED_DIR" --outfile "$GGUF_DIR/model-q4_k_m.gguf" --outtype q4_k_m
  python llama.cpp/convert_hf_to_gguf.py "$MERGED_DIR" --outfile "$GGUF_DIR/model-q5_k_m.gguf" --outtype q5_k_m
else
  echo "GGUF converter not found; skipping export."
fi

echo "Done. Artifacts:"
echo "  adapters: $ADAPTER_DIR"
echo "  merged:   $MERGED_DIR"
echo "  gguf:     $GGUF_DIR"
