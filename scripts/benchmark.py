#!/usr/bin/env python3
"""Record repeatable native benchmark samples and host/compiler metadata."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--games", type=int, default=10000)
parser.add_argument("--repeats", type=int, default=5)
parser.add_argument("--cpu", type=int)
parser.add_argument("--output", type=Path)
args = parser.parse_args()
if args.games < 1 or args.repeats < 1:
    parser.error("games and repeats must be positive")
root = Path(__file__).resolve().parents[1]
command = [str(root / "target/release/benchmark"), str(args.games)]
if args.cpu is not None:
    if not hasattr(os, "sched_getaffinity") or args.cpu not in os.sched_getaffinity(0):
        parser.error("requested CPU is not in this process's allowed affinity")
    if not shutil.which("taskset"):
        parser.error("taskset is required for --cpu")
    command = ["taskset", "-c", str(args.cpu), *command]
cpu_model = None
if Path("/proc/cpuinfo").exists():
    cpu_model = next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines()
                      if line.startswith("model name")), None)

def run(games=None):
    cmd = command if games is None else [*command[:-1], str(games)]
    return json.loads(subprocess.check_output(cmd, text=True, cwd=root))

run(1000)  # Excluded warm-up.
samples = [run() for _ in range(args.repeats)]
metrics = ["decisions_per_second", "games_per_second", "clone_ns", "restore_ns"]
report = {
    "schema": 1,
    "cpu_model": cpu_model,
    "platform": platform.platform(),
    "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
    "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
    "source_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True).strip()),
    "pinned_cpu": args.cpu,
    "games_per_sample": args.games,
    "warmup_games_excluded": 1000,
    "deck": {"Mountain": 24, "LightningBolt": 20, "GoblinPiker": 12, "HillGiant": 4},
    "deck_rules": "Freeform",
    "includes": ["episode setup", "native action generation and validation", "player observation construction", "smoke policy", "natural game endings"],
    "excludes": ["model inference", "tensor encoding", "JSON", "Python binding", "general rules and card coverage"],
    "summary": {metric: {"median": statistics.median(s[metric] for s in samples),
                          "min": min(s[metric] for s in samples), "max": max(s[metric] for s in samples)} for metric in metrics},
    "samples": samples,
}
output = json.dumps(report, indent=2) + "\n"
if args.output:
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(output)
print(output, end="")
