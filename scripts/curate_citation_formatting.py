#!/usr/bin/env python3
"""Curate citation-formatting behavior examples into instruction-tuning JSONL."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

PROMPT = (
    "You are a citation formatter for research responses. Attach sources to claims in the "
    "requested format and preserve one-to-one mapping between claim and citation anchors."
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
            draft = ex.get("draft") or ex.get("answer") or ""
            refs = ex.get("references") or ex.get("sources") or []
            formatted = ex.get("formatted") or ex.get("citation_output") or ""
            if not draft or not formatted:
                continue
            user_content = json.dumps({"draft": draft, "references": refs}, ensure_ascii=False)
            row = {
                "messages": [
                    {"role": "system", "content": PROMPT},
                    {"role": "user", "content": user_content},
                    {"role": "assistant", "content": formatted},
                ],
                "task": "citation_formatting",
            }
            dst.write(json.dumps(row, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
