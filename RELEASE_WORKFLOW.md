# Release Workflow

Use this workflow when publishing a new version (backend + frontend + GitHub release + Docker Hub images).

## One-Command Release

From repo root:

```bash
./scripts/release.sh
```

Default behavior with no version argument:
- Read latest git tag `vX.Y.Z`
- Auto-bump patch to `vX.Y.(Z+1)`
- Sync versions to:
- `Cargo.toml` (`[package].version`)
- `package.json` (root)
- `frontend/package.json`
- Build + publish everything

## Optional Version Controls

```bash
# Exact version
./scripts/release.sh v1.0.3
./scripts/release.sh 1.0.3

# Auto-bump modes
./scripts/release.sh --patch   # default
./scripts/release.sh --minor
./scripts/release.sh --major
```

## What the Script Does

1. Validate environment (`git/cargo/npm/podman/gh`) and authentication.
2. Require a clean working tree.
3. Resolve target version (explicit or auto-bumped).
4. Update versions in Rust + frontend + root package files.
5. Build frontend (`frontend/dist`).
6. Build backend binaries:
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`
7. Commit and tag:
- commit message: `release: vX.Y.Z`
- annotated tag: `vX.Y.Z`
8. Push branch and tag to GitHub.
9. Build Docker images (explicit platforms):
- `--platform linux/amd64` -> `docker.io/givenge/reader-rust:vX.Y.Z-x86_64`
- `--platform linux/arm64` -> `docker.io/givenge/reader-rust:vX.Y.Z-aarch64`
10. Verify image architecture locally.
11. Push versioned image tags.
12. Update and push rolling tags:
- `latest` (x86_64)
- `latest-aarch64` (arm64)
13. Create GitHub Release using generated notes.

## Notes

- Docker repository defaults to `docker.io/givenge/reader-rust`.
- Override repository if needed:

```bash
DOCKER_REPO=docker.io/yourname/reader-rust ./scripts/release.sh
```

- If release already exists or tag exists, script exits early to avoid accidental overwrite.
