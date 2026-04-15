#!/usr/bin/env python3
"""Curate synthesis behavior examples into instruction-tuning JSONL."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

PROMPT = (
    "You are a research synthesis assistant. Produce concise, evidence-grounded summaries, "
    "surface uncertainty, and avoid unsupported claims."
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, help="Raw JSONL path")
    parser.add_argument("--output", required=True, help="Curated JSONL path")
    args = parser.parse_args()

    in_path = Path(args.input)
    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    with in_path.open("r", encoding="utf-8") as src, out_path.open("w", encoding="utf-8") as dst:
        for line in src:
            if not line.strip():
                continue
            ex = json.loads(line)
            evidence = ex.get("evidence") or ex.get("sources") or []
            question = ex.get("query") or ex.get("goal") or ""
            synthesis = ex.get("synthesis") or ex.get("answer") or ""
            if not question or not synthesis:
                continue
            user_content = json.dumps({"query": question, "evidence": evidence}, ensure_ascii=False)
            row = {
                "messages": [
                    {"role": "system", "content": PROMPT},
                    {"role": "user", "content": user_content},
                    {"role": "assistant", "content": synthesis},
                ],
                "task": "synthesis",
            }
            dst.write(json.dumps(row, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
