# Benchmarking Guide

This project includes a three-layer benchmarking strategy for comprehensive performance measurement.

## Overview

| Layer | Tool | Measures | Best For |
|-------|------|----------|----------|
| 1 | **divan** | Wall-clock time | Fast local iteration on library code |
| 2 | **gungraun** | CPU instructions | Deterministic CI regression detection |
| 3 | **hyperfine** | End-to-end time | CLI binary benchmarks |

## Quick Start

```bash
# Generate benchmark harnesses from KDL (run after editing benchmarks.kdl)
cargo xtask gen-benchmarks

# Run all benchmarks
cargo xtask bench

# Run only divan (fast, for development)
cargo bench --bench divan_benchmarks

# Run only gungraun (deterministic, for CI)
cargo bench --bench gungraun_benchmarks

# Run only CLI benchmarks
./scripts/bench-cli.sh
```

## Unified Benchmark Definitions (KDL)

Benchmark definitions live in:

`crates/ah-ah-ah/benches/benchmarks.kdl`

Regenerate the harnesses after edits:

```bash
cargo xtask gen-benchmarks
```

This generates:

- `crates/ah-ah-ah/benches/divan_benchmarks.rs`
- `crates/ah-ah-ah/benches/gungraun_benchmarks.rs`

Do not edit the generated benchmark files directly.

## Prerequisites

### Required

- **Rust 1.80+** (for divan)

### Optional (for full benchmark suite)

- **Valgrind** (for gungraun):
  ```bash
  # macOS Intel
  brew install valgrind

  # Linux
  sudo apt install valgrind
  ```

  > **Note**: Valgrind does not support macOS ARM (M1/M2/M3). On Apple Silicon,
  > gungraun benchmarks will only run in CI on Linux runners.

- **hyperfine** (for CLI benchmarks):
  ```bash
  brew install hyperfine
  # or
  cargo install hyperfine
  ```

## Layer 1: Divan (Wall-Clock Benchmarks)

Divan provides fast, ergonomic wall-clock benchmarks for rapid development iteration.

### Location

`crates/ah-ah-ah/benches/divan_benchmarks.rs`

> Generated from `crates/ah-ah-ah/benches/benchmarks.kdl`.

### Running

```bash
# Run all divan benchmarks
cargo bench --bench divan_benchmarks

# Run benchmarks matching a pattern
cargo bench --bench divan_benchmarks -- config

# Run specific benchmark
cargo bench --bench divan_benchmarks -- load_defaults
```

### Writing Benchmarks

```rust
use std::hint::black_box;

fn main() {
    divan::main();
}

mod my_module {
    use super::*;

    // Simple benchmark
    #[divan::bench]
    fn simple_operation() -> MyType {
        black_box(my_crate::do_something())
    }

    // Parameterized benchmark
    #[divan::bench(args = [100, 1000, 10000])]
    fn varying_sizes(n: usize) -> Vec<Item> {
        black_box(my_crate::process(n))
    }

    // Benchmark with setup (setup cost excluded from measurement)
    #[divan::bench]
    fn with_setup(bencher: divan::Bencher) {
        let input = expensive_setup();
        bencher.bench_local(|| {
            black_box(my_crate::transform(&input))
        });
    }

    // Throughput measurement
    #[divan::bench]
    fn throughput(bencher: divan::Bencher) {
        let data = generate_data();
        bencher
            .counter(divan::counter::BytesCount::new(data.len()))
            .bench_local(|| {
                black_box(my_crate::process(&data))
            });
    }
}
```

### Key Features

- `#[divan::bench]` - Basic benchmark
- `#[divan::bench(args = [...])]` - Run with multiple inputs
- `divan::black_box()` - Prevent compiler optimization
- `bencher.counter()` - Measure throughput
- `divan::AllocProfiler` - Track allocations (set as global allocator)

## Layer 2: Gungraun (Instruction Count Benchmarks)

Gungraun uses Valgrind to count CPU instructions, providing **deterministic** results that are consistent across runs and machines. It evolved from iai-callgrind and provides access to all Valgrind tools.

### Location

`crates/ah-ah-ah/benches/gungraun_benchmarks.rs`

> Generated from `crates/ah-ah-ah/benches/benchmarks.kdl`.

### Running

```bash
# Run all gungraun benchmarks
cargo bench --bench gungraun_benchmarks

# Run specific benchmark
cargo bench --bench gungraun_benchmarks -- load_defaults
```

### Writing Benchmarks

```rust
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

#[library_benchmark]
fn simple_operation() -> MyType {
    black_box(my_crate::do_something())
}

// Multiple inputs for the same function
#[library_benchmark]
#[bench::small(generate_input(100))]
#[bench::medium(generate_input(1000))]
#[bench::large(generate_input(10000))]
fn varying_sizes(input: String) -> Vec<Item> {
    black_box(my_crate::process(&input))
}

library_benchmark_group!(
    name = my_benchmarks;
    benchmarks = simple_operation, varying_sizes
);

main!(library_benchmark_groups = my_benchmarks);
```

### Platform Support

| Platform | Support |
|----------|---------|
| Linux x86_64 | ✅ Full |
| Linux ARM64 | ✅ Full |
| macOS Intel (x86_64) | ✅ Full |
| macOS ARM (M1/M2/M3) | ❌ Not supported |
| Windows | ❌ Not supported |

### What Gungraun Measures

- **Instructions**: Total CPU instructions executed
- **L1 Hits**: L1 cache hits
- **L2 Hits**: L2 cache hits
- **RAM Accesses**: Main memory accesses
- **Estimated Cycles**: Approximate CPU cycles

### Why Use Instruction Counts?

Wall-clock time varies based on:
- Other processes running
- CPU frequency scaling
- Thermal throttling
- System load

Instruction counts are **deterministic** - the same code always executes the same number of instructions. This makes gungraun ideal for CI regression detection.

## Layer 3: hyperfine (CLI Benchmarks)

hyperfine measures real-world CLI performance including startup time, argument parsing, and I/O.

### Location

`scripts/bench-cli.sh`

### Running

```bash
# Full benchmark suite
./scripts/bench-cli.sh

# Quick mode (fewer iterations)
./scripts/bench-cli.sh --quick
```

### Customizing CLI Benchmarks

Edit `scripts/bench-cli.sh` to add your own commands:

```bash
# Compare different command variations
hyperfine \
    --warmup 3 \
    --export-json "$RESULTS_DIR/my-benchmark.json" \
    "$BINARY process input.txt" \
    "$BINARY process --fast input.txt" \
    "$BINARY process --parallel input.txt"

# Parameterized benchmark (varying input sizes)
hyperfine \
    --warmup 2 \
    --parameter-scan size 100 1000 -D 100 \
    "$BINARY generate --count {size}"
```

### hyperfine Options

- `--warmup N` - Run N warmup iterations before measuring
- `--min-runs N` - Minimum number of measured runs
- `--export-json FILE` - Save results as JSON
- `--export-markdown FILE` - Save results as Markdown table
- `--parameter-scan VAR MIN MAX` - Run with varying parameter values
- `--prepare CMD` - Run command before each benchmark (e.g., clear caches)

## Using xtask

The `cargo xtask bench` command provides a unified interface:

```bash
# Run all benchmarks
cargo xtask bench

# Skip specific benchmark types
cargo xtask bench --skip-divan
cargo xtask bench --skip-gungraun
cargo xtask bench --skip-cli

# Quick mode for CLI benchmarks
cargo xtask bench --quick

# Filter benchmarks by name
cargo xtask bench --filter config
```

## CI Integration

Benchmarks run automatically on push and PR via `.github/workflows/benchmarks.yml`:

- **Divan**: Runs on all PRs for quick feedback
- **Gungraun**: Runs on Linux for deterministic regression detection
- **hyperfine**: Runs in quick mode for end-to-end validation

Results are uploaded as artifacts and summarized in the GitHub Actions UI.

## Best Practices

### 1. Use `black_box()` to prevent optimization

```rust
// Bad: compiler might optimize away the result
fn benchmark() {
    my_crate::compute();
}

// Good: result is "used" so compiler can't optimize it away
fn benchmark() -> Output {
    black_box(my_crate::compute())
}
```

### 2. Separate setup from measurement

```rust
// Setup cost is excluded from measurement
#[divan::bench]
fn with_setup(bencher: divan::Bencher) {
    let input = expensive_setup();  // Not measured
    bencher.bench_local(|| {
        transform(&input)  // Measured
    });
}
```

### 3. Use realistic inputs

- Micro-benchmarks with tiny inputs may not reflect real-world performance
- Include benchmarks with production-sized data when possible

### 4. Run gungraun for CI regression detection

Wall-clock benchmarks are noisy. Gungraun provides stable, deterministic results that reliably catch performance regressions.

### 5. Close other applications for wall-clock benchmarks

When running divan or hyperfine locally, close browsers, IDEs, and other applications for more consistent results.

## Output Locations

| Benchmark | Output Location |
|-----------|-----------------|
| Divan | Terminal (no file by default) |
| Gungraun | Terminal + `target/gungraun/` |
| hyperfine | `bench-reports/*.json`, `bench-reports/*.md` |

## Troubleshooting

### "valgrind: command not found"

Install Valgrind:
```bash
# macOS Intel
brew install valgrind

# Linux
sudo apt install valgrind
```

Note: Valgrind doesn't support macOS ARM. Run gungraun in CI instead.

### "hyperfine: command not found"

```bash
brew install hyperfine
# or
cargo install hyperfine
```

### Benchmarks are too slow

- Use `cargo xtask bench --skip-gungraun` to skip instruction counting
- Use `--quick` flag for hyperfine
- Filter to specific benchmarks: `cargo bench --bench divan_benchmarks -- my_func`

### Results are inconsistent

- Close other applications
- Disable CPU frequency scaling (if possible)
- Use gungraun for deterministic results
- Increase warmup iterations

## Further Reading

- [Divan documentation](https://docs.rs/divan)
- [Gungraun documentation](https://gungraun.github.io/gungraun/latest/html/)
- [hyperfine repository](https://github.com/sharkdp/hyperfine)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
