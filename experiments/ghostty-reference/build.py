#!/usr/bin/env python3
"""Build only the approved, pinned research reference; installs into target/."""
import argparse
import copy
import os
import re
import subprocess
import tarfile
from pathlib import Path

from replay import ROOT, REVISION, encoded, sha

ARCHIVE_SHA256 = "9e3653be4beef36a408d64f6ad0b8abfacc2ee32ffe584f4c3d3ed20b2a5c487"
URL = f"https://codeload.github.com/ghostty-org/ghostty/tar.gz/{REVISION}"
ZIG_ARGS = ["zig", "build", "-Demit-lib-vt=true", "-Demit-xcframework=false",
            "-Doptimize=ReleaseSafe", "--global-cache-dir", "../zig-cache",
            "--cache-dir", "../build-cache", "--prefix", "../install"]


def upstream_files(archive, source):
    hashes = {}
    with tarfile.open(archive) as tar:
        for member in tar:
            relative = Path(*Path(member.name).parts[1:])
            path = source / relative
            if member.isfile():
                expected = sha(tar.extractfile(member).read())
                if path.is_symlink() or sha(path.read_bytes()) != expected:
                    raise RuntimeError(f"upstream source modified: {relative}")
                hashes[str(relative)] = expected
            elif member.issym() and os.readlink(path) != member.linkname:
                raise RuntimeError(f"upstream symlink modified: {relative}")
    return hashes


def dependency_inventory(work, source):
    # Inventory fetched packages, which is broader than linked runtime code.
    # Include license/manifest hashes without redistributing third-party source.
    manifests = {str(p.relative_to(source)): p.read_text() for p in source.rglob("build.zig.zon")}
    packages = []
    for archive in sorted((work / "zig-cache/p").glob("*.tar.gz")):
        licenses = {}
        with tarfile.open(archive) as tar:
            for member in tar:
                if not member.isfile():
                    continue
                relative = str(Path(*Path(member.name).parts[1:]))
                if Path(relative).name == "build.zig.zon":
                    manifests[f"cache/{archive.name}/{relative}"] = tar.extractfile(member).read().decode()
                if Path(relative).name.lower().startswith(("license", "licence", "copying", "copyright", "notice")):
                    licenses[relative] = sha(tar.extractfile(member).read())
        packages.append({"cache_archive": archive.name, "archive_sha256": sha(archive.read_bytes()),
                         "bytes": archive.stat().st_size, "license_file_sha256": licenses})
    for package in packages:
        identity = package["cache_archive"].removesuffix(".tar.gz")
        sources = []
        for name, manifest in manifests.items():
            for url, package_hash in re.findall(r'\.url\s*=\s*"([^"]+)"\s*,\s*\.hash\s*=\s*"([^"]+)"', manifest):
                if package_hash == identity:
                    sources.append({"manifest": name, "url": url, "zig_package_hash": package_hash})
        if not sources:
            raise RuntimeError(f"cannot identify fetched package: {identity}")
        package["sources"] = sources
    return {"scope": "fetched packages, not a complete linked-code SBOM or release license audit",
            "packages": packages,
            "manifest_sha256": {name: sha(text.encode()) for name, text in sorted(manifests.items())}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=Path, default=ROOT / "target/ghostty-reference")
    parser.add_argument("--archive", type=Path)
    args = parser.parse_args()
    work = args.work_dir.resolve()
    if not work.is_relative_to(ROOT / "target"):
        raise ValueError("reference builds must stay under the project's ignored target/")
    work.mkdir(parents=True, exist_ok=True)
    archive = args.archive.resolve() if args.archive else work / "source.tar.gz"
    if not archive.exists():
        subprocess.run(["curl", "-L", "--fail", "--output", str(archive), URL], check=True)
    if sha(archive.read_bytes()) != ARCHIVE_SHA256:
        raise RuntimeError("source archive hash mismatch")
    source = work / "source"
    if not source.exists():
        source.mkdir()
        with tarfile.open(archive) as tar:
            members = []
            for original in tar:
                member = copy.copy(original)
                member.name = str(Path(*Path(member.name).parts[1:]))
                members.append(member)
            tar.extractall(source, members=members, filter="data")
    upstream = upstream_files(archive, source)
    zig_version = subprocess.check_output(["zig", "version"], text=True).strip()
    if zig_version != "0.16.0":
        raise RuntimeError("this reference recipe requires the recorded stable Zig 0.16.0")
    subprocess.run(ZIG_ARGS, cwd=source, check=True)
    relative_work = work.relative_to(ROOT)
    clang_args = ["xcrun", "clang", "-std=c11", "-Wall", "-Wextra", "-Werror", "-O1", "-g",
                  "-I", str(relative_work / "source/include"), "experiments/ghostty-reference/adapter.c",
                  str(relative_work / "install/lib/libghostty-vt.a"), "-o", str(relative_work / "adapter")]
    subprocess.run(clang_args, cwd=ROOT, check=True)
    if upstream != upstream_files(archive, source):
        raise RuntimeError("build changed upstream source")
    tools = {"zig": zig_version}
    for name, cmd in {"clang": ["xcrun", "clang", "--version"], "xcode": ["xcodebuild", "-version"],
                      "sdk": ["xcrun", "--sdk", "macosx", "--show-sdk-version"],
                      "sdk_build": ["xcrun", "--sdk", "macosx", "--show-sdk-build-version"]}.items():
        tools[name] = subprocess.check_output(cmd, text=True).strip()
    manifest = {"schema_version": 1, "revision": REVISION, "archive_url": URL,
                "archive_sha256": ARCHIVE_SHA256, "upstream_regular_files": len(upstream),
                "upstream_files_manifest_sha256": sha(encoded(upstream)),
                "upstream_sources_unmodified": True, "tools": tools,
                "zig_command": ZIG_ARGS, "adapter_command": clang_args,
                "adapter_sha256": sha((work / "adapter").read_bytes()),
                "static_library_sha256": sha((work / "install/lib/libghostty-vt.a").read_bytes()),
                "build_script_sha256": sha(Path(__file__).read_bytes()),
                "adapter_source_sha256": sha((ROOT / "experiments/ghostty-reference/adapter.c").read_bytes()),
                "dynamic_libraries": subprocess.check_output(["otool", "-L", str(relative_work / "adapter")], cwd=ROOT, text=True),
                "dependencies": dependency_inventory(work, source)}
    (work / "build.json").write_bytes(encoded(manifest))
    print(f"Built pinned research reference; manifest: {work / 'build.json'}")


if __name__ == "__main__":
    main()
