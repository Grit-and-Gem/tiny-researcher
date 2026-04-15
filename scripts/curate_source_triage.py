#!/usr/bin/env python3
"""Curate source triage behavior examples into instruction-tuning JSONL."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

PROMPT = (
    "You are a research agent performing source triage. Score relevance and credibility, "
    "note conflicts, and recommend keep/discard decisions with reasons."
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
            query = ex.get("query") or ex.get("goal") or ""
            triage_input = ex.get("sources") or ex.get("context") or []
            triage_output = ex.get("triage") or ex.get("answer") or ""
            if not query or not triage_output:
                continue

            user_content = json.dumps({"query": query, "sources": triage_input}, ensure_ascii=False)
            row = {
                "messages": [
                    {"role": "system", "content": PROMPT},
                    {"role": "user", "content": user_content},
                    {"role": "assistant", "content": triage_output},
                ],
                "task": "source_triage",
            }
            dst.write(json.dumps(row, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
