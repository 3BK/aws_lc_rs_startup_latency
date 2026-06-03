# awslc-boot-bench

Micro-benchmark that isolates the **aws-lc-rs jitter-entropy initialization
latency** observed on first CSPRNG call in musl-static builds.

## Why this matters

`aws-lc-rs` seeds its CSPRNG via a jitter-entropy loop on the **first call**
to `rand::fill()`.  On systems with limited kernel entropy (containers, early
boot, VMs without `virtio-rng`), this can block for **≥ 50 ms** — a
significant hit for short-lived CLI tools or sidecar proxies.

## Build

```bash
# Prerequisites (Debian / Ubuntu)
sudo apt install musl-tools build-essential
rustup target add x86_64-unknown-linux-musl

# Build
make release        # or: cargo build --release --target x86_64-unknown-linux-musl
```

## Run

```bash
make run
# or directly:
./target/x86_64-unknown-linux-musl/release/awslc-boot-bench
```

### Sample output

```
awslc-boot-bench v0.1.0
=================================================================
Kernel entropy (if readable): 256
-----------------------------------------------------------------
Phase 0  main() entry + env probe                        0.042 ms
Phase 1  SystemRandom::new()                              0.001 ms
Phase 2  First  rand::fill(32 B)  ← SEED COST           52.318 ms
         first 8 bytes: [a3, 7f, 01, c8, 9e, 44, b2, 11]
Phase 3  Second rand::fill(32 B)                          0.003 ms
Phase 4  100× rand::fill(32 B) total                      0.048 ms
-----------------------------------------------------------------
Total wall-clock (Phases 0–4)                            52.517 ms
Amortised per-call (Phase 4)                                483 ns
=================================================================
```

## Repeat benchmark

```bash
make bench    # runs 10 iterations, extracts Phase 2 timings
```

## Static-link verification

```bash
make ldd-check
# Expected: "not a dynamic executable"
```

## Tuning kernel entropy

If the seed cost is unacceptable, ensure the VM has a hardware or
para-virtual entropy source:

| Method | How |
|---|---|
| `virtio-rng` | QEMU: `-device virtio-rng-pci` / libvirt `<rng>` element |
| `haveged` | `apt install haveged && systemctl enable haveged` |
| `rng-tools` | `apt install rng-tools` (uses RDRAND / TPM) |
| `jitterentropy` | Kernel module (loaded automatically on most distros) |

When `/proc/sys/kernel/random/entropy_avail` reports **≥ 256**,
`getrandom(2)` returns immediately and the jitter-entropy fallback
inside aws-lc is never exercised.

## License

MIT
