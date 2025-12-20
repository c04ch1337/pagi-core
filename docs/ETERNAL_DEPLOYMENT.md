# Eternal Deployment Guide (Phase 5)

This guide documents how to build and run PAGI-Core as a set of verifiable, portable binaries.

## 1) Release build (native)

```bash
cargo build --workspace --release
```

Release configuration is controlled by [`Cargo.toml`](../Cargo.toml:74).

## 2) Fully static Linux build (musl)

Use the provided helper script:

```bash
./build-musl.sh
```

Script location: [`build-musl.sh`](../build-musl.sh:1).

Artifacts are copied to `dist/musl/`.

### Verify binaries are static

```bash
ldd dist/musl/pagi-executive-engine
```

Expected output should indicate the binary is static (e.g. “not a dynamic executable”).

## 3) ExternalGateway plugin hardening

### 3.1 Seccomp sandboxing for spawned *binary* plugins (Linux)

ExternalGateway can apply a best-effort seccomp filter to spawned binary plugins.

Enable it:

```bash
export PAGI_PLUGIN_SECCOMP=true
```

Implementation is in [`apply_seccomp_deny_dangerous()`](../services/pagi-external-gateway/src/auto_discover.rs:119).

**Important:** this is a default-allow filter with a deny-list for high-risk syscalls. It is designed
to reduce blast radius without breaking typical plugins.

### 3.2 Manifest signature verification (cosign)

ExternalGateway supports opt-in verification for `manifest.toml` using `cosign verify-blob`.

Enable best-effort verification:

```bash
export PAGI_PLUGIN_VERIFY_SIGNATURES=best_effort
export PAGI_PLUGIN_COSIGN_PUBKEY=/path/to/cosign.pub
```

Enable strict verification (missing/invalid signature blocks load):

```bash
export PAGI_PLUGIN_VERIFY_SIGNATURES=strict
export PAGI_PLUGIN_COSIGN_PUBKEY=/path/to/cosign.pub
```

Signature file convention:

```text
<plugin-dir>/manifest.toml
<plugin-dir>/manifest.toml.sig
```

Verification hook: [`verify_cosign_blob()`](../services/pagi-external-gateway/src/auto_discover.rs:87).

## 4) Reproducible builds (best-effort)

Reproducibility-related flags are configured in [`./.cargo/config.toml`](../.cargo/config.toml:1).

Notes:
- Full bit-for-bit reproducibility across toolchains also depends on pinned Rust version,
  deterministic linkers, and consistent build environments.
