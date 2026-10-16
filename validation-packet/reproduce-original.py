"""Native reproduction. Run as runner on macmini with the existing builder."""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import threading
import time
import sys

assert os.uname().sysname == "Darwin"
assert subprocess.check_output(["id", "-un"], text=True).strip() == "runner"
container = "/opt/homebrew/bin/container"
original = Path("/private/tmp/yard-parallel-context.fZ4owi/contexts")
root = Path(tempfile.mkdtemp(prefix="y80-native-cause-", dir="/private/tmp"))
out = root / "evidence"
out.mkdir()
env = os.environ.copy()
env.pop("NO_COLOR", None)
env.pop("BUILDKIT_COLORS", None)
barrier = threading.Barrier(8)
rows = []

def capture(label, command):
    p = subprocess.run(command, env=env, capture_output=True, timeout=30)
    (out / (label + ".stdout")).write_bytes(p.stdout)
    (out / (label + ".stderr")).write_bytes(p.stderr)
    if p.returncode:
        raise RuntimeError((label, p.returncode))
    return p.stdout

capture("builder-before", [container, "list", "--all", "--format", "json"])
capture("receiver-before", [container, "exec", "buildkit", "sh", "-c",
    "find /var/lib/container-builder-shim/fssync -maxdepth 1 -name '*.tar' -exec ls -l {} \\;"])
for i in range(8):
    context = root / "contexts" / str(i)
    shutil.copytree(original / str(i), context, copy_function=shutil.copyfile)
    rows.append({"case": i, "kind": "large" if i < 2 else "small-control",
        "context": str(context), "tag": f"y80-native-cause-{os.getpid()}-{i}"})
equal_metadata = "--equal-metadata" in sys.argv[1:]
distinct_metadata = "--distinct-metadata" in sys.argv[1:]
assert not (equal_metadata and distinct_metadata)
metadata_timestamp = None
if equal_metadata or distinct_metadata:
    metadata_timestamp = int(time.time()) - 60
    for row in rows:
        payload = Path(row["context"]) / "payload"
        stamp = metadata_timestamp + (row["case"] if distinct_metadata else 0)
        for p in [*payload.rglob("*"), payload]:
            os.utime(p, (stamp, stamp))
    # Darwin moves birth time back when setting an earlier modification time.
    paired = []
    for row in rows[:2]:
        payload = Path(row["context"]) / "payload"
        paired.append([(str(p.relative_to(payload)), int(p.stat().st_birthtime),
            int(p.stat().st_mtime), p.stat().st_mode, p.stat().st_size)
            for p in sorted([*payload.rglob("*"), payload])])
    if equal_metadata:
        assert paired[0] == paired[1], "large contexts do not have matched hashed metadata"
    else:
        assert paired[0] != paired[1], "large contexts unexpectedly share hashed metadata"
manifest = {}
for row in rows:
    context = Path(row["context"])
    manifest[str(row["case"])] = {
        str(p.relative_to(context)): {"size": p.stat().st_size,
            "mtime_ns": p.stat().st_mtime_ns, "birthtime": p.stat().st_birthtime,
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
        for p in sorted(context.rglob("*")) if p.is_file()}
(out / "inputs.json").write_text(json.dumps(manifest, indent=2) + "\n")
(out / "plan.json").write_text(json.dumps(rows, indent=2) + "\n")
def build(row):
    command = [container, "build", "--file", row["context"] + "/.yard/Dockerfile",
        "--tag", row["tag"], row["context"]]
    barrier.wait(timeout=30)
    started = time.monotonic()
    with (out / f'{row["case"]}.log').open("wb") as log:
        try:
            p = subprocess.run(command, env=env, stdin=subprocess.DEVNULL,
                stdout=log, stderr=subprocess.STDOUT, timeout=180)
            result = {**row, "command": command, "exit": p.returncode,
                "seconds": time.monotonic() - started}
        except subprocess.TimeoutExpired:
            result = {**row, "command": command, "timeout": 180,
                "seconds": time.monotonic() - started}
    (out / f'{row["case"]}.json').write_text(json.dumps(result, indent=2) + "\n")
    return result
with ThreadPoolExecutor(max_workers=8) as pool:
    results = list(pool.map(build, rows))
capture("receiver-after", [container, "exec", "buildkit", "sh", "-c",
    "find /var/lib/container-builder-shim/fssync -maxdepth 1 -name '*.tar' -exec ls -l {} \\;"])
capture("builder-after", [container, "list", "--all", "--format", "json"])
# Every build has finished. Only successful images with these exact owned tags are retired.
for result in results:
    if result.get("exit") == 0:
        p = subprocess.run([container, "image", "delete", result["tag"]], env=env,
            capture_output=True, text=True, timeout=30)
        result["owned_tag_retirement"] = {"exit": p.returncode,
            "stdout": p.stdout, "stderr": p.stderr}
report = {"root": str(root), "concurrency": 8, "cache": "default",
    "equal_metadata": equal_metadata, "distinct_metadata": distinct_metadata,
    "metadata_timestamp": metadata_timestamp,
    "TMPDIR": os.environ.get("TMPDIR"), "results": results}
(out / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2), flush=True)
