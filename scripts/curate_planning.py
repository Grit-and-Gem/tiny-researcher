#!/usr/bin/env python3
"""Curate planning behavior examples into instruction-tuning JSONL."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

PROMPT = (
    "You are a research agent planner. Break the user goal into an ordered plan "
    "with explicit milestones and immediate next action."
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
            user_goal = ex.get("goal") or ex.get("query") or ""
            plan = ex.get("plan") or ex.get("answer") or ""
            if not user_goal or not plan:
                continue
            row = {
                "messages": [
                    {"role": "system", "content": PROMPT},
                    {"role": "user", "content": user_goal},
                    {"role": "assistant", "content": plan},
                ],
                "task": "planning",
            }
            dst.write(json.dumps(row, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
