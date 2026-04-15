#!/usr/bin/env python3
"""Minimal QLoRA fine-tune entrypoint for research-agent behaviors."""

from __future__ import annotations

import argparse
from pathlib import Path

import torch
from datasets import load_dataset
from peft import LoraConfig
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig, TrainingArguments
from trl import SFTTrainer


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--model", required=True)
    p.add_argument("--train-file", required=True)
    p.add_argument("--output-adapter-dir", required=True)
    p.add_argument("--batch-size", type=int, default=1)
    p.add_argument("--grad-accum", type=int, default=16)
    p.add_argument("--context-length", type=int, default=2048)
    p.add_argument("--save-steps", type=int, default=200)
    p.add_argument("--max-steps", type=int, default=1000)
    p.add_argument("--learning-rate", type=float, default=2e-4)
    return p.parse_args()


def format_chat(example: dict) -> dict:
    text = ""
    for m in example["messages"]:
        text += f"<{m['role']}>\n{m['content']}\n"
    return {"text": text}


def main() -> None:
    args = parse_args()
    out = Path(args.output_adapter_dir)
    out.mkdir(parents=True, exist_ok=True)

    quant = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
        bnb_4bit_compute_dtype=torch.bfloat16 if torch.cuda.is_available() else torch.float16,
    )

    model = AutoModelForCausalLM.from_pretrained(args.model, quantization_config=quant, device_map="auto")
    tokenizer = AutoTokenizer.from_pretrained(args.model, use_fast=True)
    tokenizer.pad_token = tokenizer.eos_token

    ds = load_dataset("json", data_files=args.train_file, split="train")
    ds = ds.map(format_chat)

    lora = LoraConfig(
        r=16,
        lora_alpha=32,
        lora_dropout=0.05,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "up_proj", "down_proj", "gate_proj"],
        task_type="CAUSAL_LM",
    )

    train_args = TrainingArguments(
        output_dir=str(out),
        per_device_train_batch_size=args.batch_size,
        gradient_accumulation_steps=args.grad_accum,
        learning_rate=args.learning_rate,
        max_steps=args.max_steps,
        save_steps=args.save_steps,
        logging_steps=10,
        bf16=torch.cuda.is_available(),
        fp16=not torch.cuda.is_available(),
        gradient_checkpointing=True,
        report_to="none",
    )

    trainer = SFTTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=ds,
        peft_config=lora,
        max_seq_length=args.context_length,
        args=train_args,
        dataset_text_field="text",
    )
    trainer.train()
    trainer.model.save_pretrained(out)
    tokenizer.save_pretrained(out)


if __name__ == "__main__":
    main()
