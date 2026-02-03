# Solana Shmem Bridge

A Geyser plugin and utilities for ultra-low-latency transaction delivery via shared memory.

## Features
- Zero-Copy Design: Data is not copied unnecessarily; it is memory‑mapped directly.
- Lock-Free Concurrency: No Mutex usage, only atomic operations.
- HFT Ready: Built for local arbitrage and liquidation bots.

## Binaries
- `init`: creates and zeroes the shared memory segment.
- `consumer`: reads slots and prints signatures/latency.

## Installation
1. Build the project:
   `cargo build --release`

## Usage
1. Run `init` to create the shared memory segment:
   `cargo run --release --bin init`
2. Start the validator with the Geyser plugin configured.
3. Run `consumer` to read transactions:
   `cargo run --release --bin consumer`

## Configuration
The plugin uses a shared memory segment named `solana_bridge` on macOS and `/solana_bridge` on Linux.
Update your validator Geyser config to load the plugin and ensure the segment is initialized before startup.

To avoid hardcoding `libpath`, generate `config.json` with the helper script:
`bash scripts/gen_config.sh`

## Requirements
- Rust toolchain
- Solana validator with Geyser support

## Benchmarks
| Method | Avg Latency | Speedup |
| --- | --- | --- |
| Yellowstone gRPC (Local) | ~1,200,000 ns | 1x |
| Shared-Memory Bridge (Ours) | ~13,000 ns | ~92x |

Note: Tested on Apple M2 Pro, 32GB RAM, Solana v3.0.13.

## Notes
- On Linux the `os_id` is `"/solana_bridge"`, on macOS it is `"solana_bridge"`.
