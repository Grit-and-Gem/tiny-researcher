# tiny-researcher

Local-first research loop scaffold with an orchestrator, model assets, and configurable execution pipelines.

## Project layout

- `orchestrator/` — Rust binary crate that coordinates pipeline steps.
- `models/` — quantized model files, adapters, and tokenizers.
- `python_runtime/` — optional local model-server and utility helpers.
- `configs/` — hardware profile TOML files.
- `pipelines/` — YAML/TOML workflow definitions for the research loop.
- `data/` — `raw/`, `processed/`, and `eval/` datasets/artifacts.
- `docs/` — setup docs and policy docs.
- `scripts/` — benchmark, dataset preparation, and evaluation scripts.

## 1) No external API policy

This repository is designed to run **without calling third-party hosted model APIs**.

- All inference is expected to run on local hardware (GPU/CPU).
- Pipelines should avoid network-dependent inference endpoints.
- Documentation and examples must preserve a **no external API** guarantee unless explicitly marked otherwise.

See: `docs/no_api_guarantee.md`.

## 2) Supported VRAM tiers

The initial target tiers are:

- **4 GB VRAM** (e.g., GTX 1650 class) using compact quantization and conservative context.
- **CPU fallback** for environments without compatible GPUs.
- **Higher VRAM GPUs (6/8/12+ GB)** are expected to work with larger quantizations/context via additional profiles.

Included starter profiles:

- `configs/gtx1650_4gb.toml`
- `configs/cpu_fallback.toml`

## 3) Exact startup command

From repository root:

```bash
cargo run --manifest-path orchestrator/Cargo.toml
```

## 4) Model format expectations

Model artifacts placed under `models/` should be in one of these compatible formats:

- **GGUF** (preferred for modern llama.cpp-compatible runtimes)
- **GGML** (legacy compatibility)
- **exllama-compatible** quantized checkpoints/layouts

Tokenizers and adapter assets should be colocated with each model family under `models/`.
