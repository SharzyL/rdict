# RDict - AI-Powered Dictionary

A lightweight, fast dictionary application powered by AI (OpenAI-compatible APIs) with a native GUI built in Rust using egui.

## Build & Run

### Prerequisites

- Rust 1.91 or later
- For Nix users: just run `nix develop` to get the development environment

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

Or with custom config file:

```bash
cargo run -- --config /path/to/config.toml
```

### Command Line Options

- `--config <PATH>` - Path to config file (defaults to `~/.config/rdict/config.toml`)
- `--debug` - Enable debug logging
- `--quiet` - Suppress info and debug logs, only show warnings and errors

## Configuration

Configuration is stored in `~/.config/rdict/config.toml`:

```toml
[api]
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
model = "claude-haiku-4-5"
response_language = "Chinese"
```

### Configuration Options

- `api.base_url` - OpenAI-compatible API endpoint (default: `https://api.openai.com/v1`)
- `api.api_key` - Your API key for authentication
- `api.model` - Model to use for dictionary lookups (default: `claude-haiku-4-5`)
- `api.response_language` - Language for dictionary responses (default: `Chinese`). Can be set to any language (e.g., `English`, `Japanese`, `Spanish`, etc.)

## Development

### Using Nix

```bash
nix develop           # Enter development shell
nix build            # Build the package
nix fmt              # Format code
```

### Code Formatting

```bash
cargo fmt
```

## License

This project is licensed under the WTFPL (Do What The Fuck You Want To Public License).
See the [LICENSE](LICENSE) file for details.
