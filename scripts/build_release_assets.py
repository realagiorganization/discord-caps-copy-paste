#!/usr/bin/env python3
"""Build Debian and Homebrew release assets for discord-caps-copy-paste."""

from __future__ import annotations

import argparse
import hashlib
import shutil
import subprocess
import tempfile
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parents[1]
PACKAGE_NAME = "discord-caps-copy-paste"
PACKAGE_DESCRIPTION = (
    "Launch a fresh Codex CLI prompt in a random installed terminal and attach it to Discord via Tether."
)
PACKAGE_HOMEPAGE = "https://github.com/realagiorganization/discord-caps-copy-paste"
PACKAGE_MAINTAINER = "RealAGI Organization <opensource@realagiorganization.invalid>"


def run(cmd: list[str], *, cwd: Path | None = None) -> None:
    subprocess.run(cmd, cwd=str(cwd) if cwd else None, check=True)


def cargo_package_metadata() -> dict[str, object]:
    payload = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    package = payload.get("package")
    if not isinstance(package, dict):
        raise SystemExit("Cargo.toml is missing [package] metadata")
    return package


def package_version() -> str:
    version = cargo_package_metadata().get("version")
    if not isinstance(version, str) or not version.strip():
        raise SystemExit("Cargo.toml package.version is missing")
    return version.strip()


def validate_release_tag(release_tag: str | None, version: str) -> str:
    if release_tag is None:
        return f"v{version}"
    expected = release_tag.removeprefix("v")
    if expected != version:
        raise SystemExit(
            f"release tag/version mismatch: tag {release_tag!r} expects {expected!r}, "
            f"but Cargo.toml has {version!r}"
        )
    return release_tag


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def debian_version(version: str) -> str:
    return version.replace("-rc", "~rc").replace("-beta", "~beta").replace("-alpha", "~alpha")


def debian_architecture() -> str:
    machine = subprocess.run(
        ["uname", "-m"],
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()
    mapping = {
        "x86_64": "amd64",
        "aarch64": "arm64",
        "armv7l": "armhf",
    }
    return mapping.get(machine, machine)


def ruby_class_name(raw_name: str) -> str:
    return "".join(part.capitalize() for part in raw_name.replace("_", "-").split("-") if part)


def source_archive_name(version: str) -> str:
    return f"{PACKAGE_NAME}-{version}-source.tar.gz"


def deb_asset_name(version: str, arch: str) -> str:
    return f"{PACKAGE_NAME}_{debian_version(version)}_{arch}.deb"


def formula_name() -> str:
    return f"{PACKAGE_NAME}.rb"


def build_binary() -> Path:
    run(["cargo", "build", "--locked", "--release"], cwd=ROOT)
    binary = ROOT / "target" / "release" / PACKAGE_NAME
    if not binary.exists():
        raise SystemExit(f"missing built binary: {binary}")
    return binary


def build_source_archive(version: str, output_path: Path) -> Path:
    prefix = f"{PACKAGE_NAME}-{version}/"
    run(
        [
            "git",
            "archive",
            "--format=tar.gz",
            f"--prefix={prefix}",
            "-o",
            str(output_path),
            "HEAD",
        ],
        cwd=ROOT,
    )
    return output_path


def create_formula(source_archive: Path, release_tag: str, output_path: Path) -> Path:
    class_name = ruby_class_name(PACKAGE_NAME)
    asset_url = (
        f"{PACKAGE_HOMEPAGE}/releases/download/{release_tag}/{source_archive.name}"
    )
    content = "\n".join(
        [
            f"class {class_name} < Formula",
            f'  desc "{PACKAGE_DESCRIPTION}"',
            f'  homepage "{PACKAGE_HOMEPAGE}"',
            f'  url "{asset_url}"',
            f'  sha256 "{sha256(source_archive)}"',
            '  license "MIT"',
            '  depends_on "rust" => :build',
            "",
            "  def install",
            '    system "cargo", "install", *std_cargo_args(path: "."), "--locked"',
            "  end",
            "",
            "  test do",
            f'    assert_match "Prompt", shell_output("#{{bin}}/{PACKAGE_NAME} --help")',
            "  end",
            "end",
            "",
        ]
    )
    output_path.write_text(content, encoding="utf-8")
    return output_path


def create_deb(binary_path: Path, version: str, output_path: Path) -> Path:
    arch = debian_architecture()
    deb_version = debian_version(version)
    with tempfile.TemporaryDirectory(prefix=f"{PACKAGE_NAME}-deb-") as temp_root:
        pkg_root = Path(temp_root) / PACKAGE_NAME
        debian_dir = pkg_root / "DEBIAN"
        bin_dir = pkg_root / "usr" / "bin"
        doc_dir = pkg_root / "usr" / "share" / "doc" / PACKAGE_NAME
        debian_dir.mkdir(parents=True, exist_ok=True)
        bin_dir.mkdir(parents=True, exist_ok=True)
        doc_dir.mkdir(parents=True, exist_ok=True)

        shutil.copy2(binary_path, bin_dir / PACKAGE_NAME)
        shutil.copy2(ROOT / "README.md", doc_dir / "README.md")
        shutil.copy2(ROOT / "LICENSE", doc_dir / "LICENSE")

        control = "\n".join(
            [
                f"Package: {PACKAGE_NAME}",
                f"Version: {deb_version}",
                "Section: utils",
                "Priority: optional",
                f"Architecture: {arch}",
                f"Maintainer: {PACKAGE_MAINTAINER}",
                "Depends: bash",
                f"Homepage: {PACKAGE_HOMEPAGE}",
                "Description: Discord-driven Codex session launcher for Tether",
                " Narrow Rust launcher that resolves a prompt from CLI, env, or clipboard,",
                " opens a fresh Codex session in a supported terminal, waits for Tether",
                " discovery, and attaches the new external session to Discord.",
                "",
            ]
        )
        (debian_dir / "control").write_text(control, encoding="utf-8")

        run(
            [
                "dpkg-deb",
                "--root-owner-group",
                "--build",
                str(pkg_root),
                str(output_path),
            ]
        )
    return output_path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release-tag", default="", help="Release tag, for example v0.1.1")
    parser.add_argument("--dist-dir", type=Path, default=ROOT / "dist")
    args = parser.parse_args()

    version = package_version()
    release_tag = validate_release_tag(args.release_tag or None, version)
    dist_dir = args.dist_dir.resolve()
    dist_dir.mkdir(parents=True, exist_ok=True)

    binary = build_binary()
    archive_path = build_source_archive(version, dist_dir / source_archive_name(version))
    formula_path = create_formula(archive_path, release_tag, dist_dir / formula_name())
    deb_path = create_deb(binary, version, dist_dir / deb_asset_name(version, debian_architecture()))

    print(archive_path)
    print(formula_path)
    print(deb_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
