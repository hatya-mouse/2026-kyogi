#!/usr/bin/env python3
"""Start the manager at each scheduled practice-match start time."""

from __future__ import annotations

import argparse
import datetime as dt
import subprocess
import time


def next_start(now: dt.datetime, minutes: tuple[int, ...]) -> dt.datetime:
    for minute in minutes:
        candidate = now.replace(second=0, microsecond=0, minute=minute)
        if candidate > now:
            return candidate

    return (now + dt.timedelta(hours=1)).replace(
        minute=minutes[0], second=0, microsecond=0
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True)
    parser.add_argument("--manager", default="cargo")
    parser.add_argument("--once", action="store_true")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()

    command = (
        [args.manager]
        if args.manager != "cargo"
        else ["cargo", "run", "-p", "manager", "--quiet", "--"]
    )
    command += ["--config", args.config]
    if args.verbose:
        command.append("--verbose")

    starts = (12, 27, 42)
    while True:
        now = dt.datetime.now().astimezone()
        start = next_start(now, starts)
        wait_seconds = (start - now).total_seconds()
        print(f"Next practice match: {start.isoformat()}", flush=True)
        time.sleep(wait_seconds)

        print(f"Starting manager: {' '.join(command)}", flush=True)
        result = subprocess.run(command, check=False)
        print(f"Manager exited with status {result.returncode}.", flush=True)

        if args.once:
            return


if __name__ == "__main__":
    main()
