# Local Reasoning Model Plan (1B–3B class)

This document defines base models, quantization targets, fine-tune output layout, and expected memory footprints for tiny-researcher.

## 1) Base models (small, local-reasoning friendly)

We target one primary and one fallback model in the 1B–3B range:

1. **Qwen2.5-1.5B-Instruct** (primary)
   - Good instruction-following quality for its size.
   - Strong enough for planning + synthesis tasks under QLoRA.
2. **Llama-3.2-3B-Instruct** (optional higher-capacity fallback)
   - Better headroom for source triage and citation consistency.
   - Slightly higher memory and slower inference than 1.5B.

> Start with Qwen2.5-1.5B-Instruct for fast iteration; move to 3B only if quality is insufficient.

## 2) Quantization targets

### Train-time (fine-tune)
- **QLoRA 4-bit** (NF4 + double quantization + bfloat16 compute where available).
- LoRA adapters trained on top of frozen quantized base model.

Recommended defaults:
- `load_in_4bit=True`
- `bnb_4bit_quant_type=nf4`
- `bnb_4bit_use_double_quant=True`
- `bnb_4bit_compute_dtype=bfloat16` (fallback to `float16` on older GPUs)

### Inference-time (GGUF)
Export merged checkpoints to GGUF and provide:
- **Q4_K_M** (default low-memory profile)
- **Q5_K_M** (higher quality if memory allows)

Use Q4 on 4GB-class VRAM and CPU fallback; prefer Q5 for 8GB+ when latency is acceptable.

## 3) Dataset behavior tracks

Training data is curated into four behavior-specific tracks:

1. **Planning**: decomposition, milestone creation, next-action selection.
2. **Source triage**: credibility/risk checks, relevance scoring, conflict detection.
3. **Synthesis**: evidence-grounded answer drafting with uncertainty notes.
4. **Citation formatting**: normalized citations and source mapping to claims.

Scripts for these tracks live under `scripts/` and produce JSONL instruction pairs.

## 4) Reproducible fine-tune entrypoint

Use `scripts/run_finetune.sh` to run end-to-end fine-tuning with configurable:

- batch size,
- gradient accumulation,
- context length,
- checkpoint interval.

The script emits artifacts into `models/finetuned/`:

- `adapters/<run_name>/` (LoRA adapters),
- `merged/<run_name>/` (merged fp16/bf16 checkpoint),
- `gguf/<run_name>/` (Q4/Q5 GGUF exports, when converter is available).

## 5) Expected memory footprints

Approximate profiles for QLoRA training + inference planning.

| Profile | Hardware target | Train-time expectation | Inference expectation |
|---|---|---|---|
| `gtx1650_4gb` | 4GB VRAM + system RAM | 1.5B only, 4-bit QLoRA, micro-batch 1, grad accum 16–32, context 1024–1536, gradient checkpointing enabled. | Q4 GGUF preferred; Q5 may OOM on long context. |
| `8gb` | 8GB VRAM | 1.5B comfortably; 3B feasible with micro-batch 1 and accum tuning, context 2048 typical. | Q4 or Q5 GGUF both practical; Q5 recommended for better quality. |
| `cpu_fallback` | No usable CUDA VRAM | Fine-tune generally impractical (too slow); use pre-trained + adapter merge done elsewhere. | Q4 GGUF via llama.cpp with reduced threads/context for responsiveness. |

### Ballpark memory numbers (inference)

- **1.5B Q4 GGUF**: ~1.2–1.8 GB model memory + KV cache overhead.
- **1.5B Q5 GGUF**: ~1.6–2.3 GB + KV cache.
- **3B Q4 GGUF**: ~2.3–3.2 GB + KV cache.
- **3B Q5 GGUF**: ~3.0–4.2 GB + KV cache.

Actual usage varies by runtime, context length, batch size, and offload strategy.

## 6) Artifact convention

All fine-tune outputs must remain under:

```text
models/finetuned/
  adapters/<run_name>/
  merged/<run_name>/
  gguf/<run_name>/
```

This keeps experiments reproducible and easy to diff.
