# Docker Release Runbook (Podman)

This runbook is the default flow for publishing `reader-rust` images to Docker Hub.

Repository: `docker.io/givenge/reader-rust`

## Release Tags

- x86_64 image tag: `vX.Y.Z-x86_64`
- arm64 image tag: `vX.Y.Z-aarch64`
- rolling tags:
- `latest` -> x86_64 image
- `latest-aarch64` -> arm64 image

## Prerequisites

- `podman` is available and logged in to Docker Hub:

```bash
podman login docker.io
```

- Build artifacts are generated on host first (Dockerfiles do not compile Rust in-container).

## Standard Commands

Run from repo root.

1. Set release version:

```bash
export TAG=v1.0.2
```

2. Build frontend:

```bash
cd frontend && npm run build && cd ..
```

3. Build Rust binaries:

```bash
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
```

4. Build images:

```bash
podman build --platform linux/amd64 -t docker.io/givenge/reader-rust:${TAG}-x86_64 -f Dockerfile.x86 .
podman build --platform linux/arm64 -t docker.io/givenge/reader-rust:${TAG}-aarch64 -f Dockerfile .
```

5. Verify architecture:

```bash
podman image inspect docker.io/givenge/reader-rust:${TAG}-x86_64 --format '{{.Architecture}} {{.Os}}'
podman image inspect docker.io/givenge/reader-rust:${TAG}-aarch64 --format '{{.Architecture}} {{.Os}}'
```

Expected:
- `${TAG}-x86_64` => `amd64 linux`
- `${TAG}-aarch64` => `arm64 linux`

6. Push versioned tags:

```bash
podman push docker.io/givenge/reader-rust:${TAG}-x86_64
podman push docker.io/givenge/reader-rust:${TAG}-aarch64
```

7. Update rolling tags:

```bash
podman tag docker.io/givenge/reader-rust:${TAG}-x86_64 docker.io/givenge/reader-rust:latest
podman tag docker.io/givenge/reader-rust:${TAG}-aarch64 docker.io/givenge/reader-rust:latest-aarch64

podman push docker.io/givenge/reader-rust:latest
podman push docker.io/givenge/reader-rust:latest-aarch64
```

## Optional: Multi-Arch Unified Tag

If you need `docker.io/givenge/reader-rust:${TAG}` as a multi-arch manifest:

```bash
podman manifest create docker.io/givenge/reader-rust:${TAG}
podman manifest add docker.io/givenge/reader-rust:${TAG} docker.io/givenge/reader-rust:${TAG}-x86_64
podman manifest add docker.io/givenge/reader-rust:${TAG} docker.io/givenge/reader-rust:${TAG}-aarch64
podman manifest push --all docker.io/givenge/reader-rust:${TAG}
```

## Quick Verification

```bash
podman manifest inspect docker.io/givenge/reader-rust:${TAG}
podman search docker.io/givenge/reader-rust
```
