#!/usr/bin/env python3
"""Create an unpublished source candidate with exact byte provenance, excluding ignored data."""
import hashlib
import json
import subprocess
import sys
import zipfile
from pathlib import Path

root = Path(__file__).resolve().parent.parent
if len(sys.argv) != 2:
    raise SystemExit("Use: python3 scripts/source-snapshot.py <new output.zip>")
output = Path(sys.argv[1]).resolve()
if output.exists():
    raise SystemExit("Refusing to replace an existing snapshot")
paths = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=root).decode().split("\0")
files = []
for name in sorted(set(filter(None, paths))):
    path = root / name
    if not path.exists():  # tracked deletion
        continue
    if path.is_symlink() or not path.is_file():
        raise SystemExit("Unexpected non-regular source input")
    if name == "CANDIDATE_SOURCE.json":
        raise SystemExit("Candidate manifest already exists in checkout")
    if any(part in {".git", "node_modules", "target", "local-data", "evidence"} for part in Path(name).parts) or path.name == "config.json" or path.name.startswith(".env"):
        raise SystemExit("Unexpected private/build input in source list")
    content = path.read_bytes()
    files.append((name, content, path.stat().st_mode))
records = [{"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()} for name, data, _ in files]
canonical = json.dumps(records, ensure_ascii=False, separators=(",", ":")).encode()
manifest = {
    "schema": 1,
    "status": "unpublished working-tree candidate; not a tagged release",
    "version": json.loads((root / "package.json").read_text())["version"],
    "baseCommit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root).decode().strip(),
    "sourceManifestSha256": hashlib.sha256(canonical).hexdigest(),
    "files": records,
}
output.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for name, data, mode in files:
        info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = (mode & 0xFFFF) << 16
        archive.writestr(info, data)
    archive.writestr("CANDIDATE_SOURCE.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
print(json.dumps({"path": str(output), "sha256": hashlib.sha256(output.read_bytes()).hexdigest(), "bytes": output.stat().st_size, "sourceManifestSha256": manifest["sourceManifestSha256"], "files": len(records)}))
