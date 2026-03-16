# Examples

## count

Reads stdin, counts Claude tokens, prints the count.

```bash
echo -n "Hello, world!" | cargo run --example count
# 4

cat src/lib.rs | cargo run --example count
# ~150 (varies with file content)
```

Used by `scripts/gen-token-fixtures.sh` to compare ah-ah-ah counts against
the live Anthropic API.
