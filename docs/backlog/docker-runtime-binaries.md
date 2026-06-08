# Medium, P1: Docker image bundles proxy engines

### Status

Planned

### Goal

Include pre-installed xray and sing-box binaries in the Docker image so
container users do not need to install or mount them separately.

### Current behavior

The image currently depends on external engine availability (manual install,
mount, or host-provided binary).

### Changes required

- Pick a base-image strategy: distro packages, release archive download during
  build, or multi-stage copy from upstream images.
- Keep image size reasonable and verify binary integrity (checksums/signatures
  where possible).
- Ensure both engines run correctly as the container runtime user.

### Possible root cause

Packaging currently treats engine binaries as host/runtime prerequisites instead
of Docker image assets.

### Verification

- Build the image for supported architectures.
- Run `xrat status`/`xrat connect` in-container and confirm both engine paths
  resolve without bind mounts.

### Open decisions

- Which source is most reliable for multi-arch musl/glibc compatibility?
- Should image variants exist (`minimal` vs `with-engines`)?
