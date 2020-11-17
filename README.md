# Taurus - A Concurrent Gemini Server

## Building

```sh
cargo build --release
cp target/release/taurus /usr/bin/taurus
```

## Usage

By default, taurus looks for a configuration file at `/etc/taurus/taurus.toml`. Fields that can be configured are defined in `src/config.rs`.

An example config might look like this:

```toml

# Default gemini port is 1965
port = 1965

# Your TLS certificate
certificate_file = "/etc/taurus/identity.pfx"

# Must match with the export password of the generated certificate
certificate_password = "mysecretpassword"

# Your gemini files
static_root = "/var/www/gemini"
```

You will need a TLS certificate in order to use taurus. To generate one, take a look at the section below.

## Generating a test-certificate

At the current state of the project, you need to generate a server certificate by hand. Take a look at `contrib/generate_cert.sh`, and run it.

## Testing

There is a diagnostics script at `contrib/diagnostics.py` that can be used to test the functionality of taurus.
