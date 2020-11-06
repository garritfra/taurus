# Taurus - A Concurrent Gemini Server

## Building

```sh
cargo build --release
cp target/release/taurus /usr/bin/taurus
```

## Generating a test-certificate

At the current state of the project, you need to generate a server certificate by hand. Take a look at `contrib/generate_cert.sh`, and run it.

## Testing

There is a diagnostics script at `contrib/diagnostics.py` that can be used to test the functionality of taurus.
