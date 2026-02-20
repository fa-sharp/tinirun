# Axum Web Service Template

A production-ready template for building web services with Rust and Axum.

## Features

- **Axum** - Fast and ergonomic web framework
- **Configuration Management** - Environment-based config with `figment`
- **Structured Logging** - JSON logging in production with `tracing`
- **Graceful Shutdown** - Handles SIGTERM and SIGINT signals
- ️**Plugin Architecture** - Modular app initialization with `axum-app-wrapper`
- **Optional OpenAPI** - API documentation with `aide` (optional)

## Usage

### Using cargo-generate

Install cargo-generate if you haven't already:

```bash
cargo install cargo-generate
```

Generate a new project from this template:

```bash
cargo generate --git <your-template-repo-url>
```

You'll be prompted for:
- **Project name**: The name of your new project
- **Project description**: A brief description
- **Environment variable prefix**: Prefix for env vars (e.g., `APP` for `APP_HOST`, `APP_PORT`)
- **Default port**: The server's default port
- **Default log level**: trace, debug, info, warn, or error
- **Include aide**: Whether to include OpenAPI documentation support

### Manual Setup

1. Clone or download this repository
2. Update `Cargo.toml` with your project name and details
3. Copy `.env.example` to `.env` and configure your environment variables
4. Run `cargo build`

## Configuration

Configuration is loaded from environment variables. The prefix is configurable during template generation.

Example with `APP` prefix:

```bash
# Required
APP_API_KEY=your-secret-key

# Optional (defaults shown)
APP_HOST=127.0.0.1
APP_PORT=8080
APP_LOG_LEVEL=info
```

In development, you can use the `.env` file.

## Project Structure

```
.
├── src/
│   ├── main.rs       # Entry point, server setup
│   ├── lib.rs        # App initialization
│   ├── config.rs     # Configuration management
│   └── state.rs      # Application state
├── Cargo.toml        # Dependencies
└── .env.example      # Example environment variables
```

## Development

```bash
# Run in development mode (loads .env file)
cargo run

# Build for production
cargo build --release
```

## License

MIT License
