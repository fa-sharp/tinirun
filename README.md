# tinirun

A code execution service that can run arbitrary code snippets in ephemeral Docker containers. Submit code in a supported language via HTTP and get back streaming stdout/stderr/result output. Supports one-off execution and persistent named functions with pre-built images. See `/demo` folder for a demo application.

⚠️ This service is mostly designed for local/development use and small self-hosted environments. It is not a scalable solution for a production environment. The containers are isolated as much as possible, but running untrusted code always comes with risks.

## Quick Start

```bash
docker compose up
```

This starts a Redis-compatible store (Valkey), the tinirun server on port 8080, and a demo web UI on port 3000. The default API key is `api-key-123`.

API docs are available at `http://localhost:8080/api/docs`.

## Configuration

All server config uses environment variables prefixed with `RUNNER_`:

| Variable | Required | Default | Description |
|---|---|---|---|
| `RUNNER_REDIS_URL` | Yes | — | Redis connection URL |
| `RUNNER_API_KEY` | Yes | — | API key for `X-Runner-Api-Key` header |
| `RUNNER_HOST` | No | `127.0.0.1` | Bind address |
| `RUNNER_PORT` | No | `8082` | Bind port |
| `RUNNER_LOG_LEVEL` | No | `warn` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `RUNNER_CLEANUP_INTERVAL` | No | `300` | Seconds between Docker image cleanup runs |

Docker connectivity uses standard Docker SDK environment variables (`DOCKER_HOST`, `DOCKER_TLS_VERIFY`, etc.).

## Authentication

All API routes require the `X-Runner-Api-Key` header.

## API Routes

### Code Execution

**`POST /api/code/run`** — Run a one-off code snippet.

Request body:
```json
{
  "code": "print('hello')",
  "lang": "python",
  "dependencies": [],
  "files": [],
  "timeout": 60,
  "mem_limit_mb": 256,
  "cpu_limit": 0.5
}
```

Supported languages: `bash`, `go`, `javascript`, `python`, `rust`, `typescript`

### Functions

See OpenAPI docs for creating and running persisted functions.

### Documentation

**`GET /api/openapi.json`** — OpenAPI spec (no auth required)

**`GET /api/docs`** — Swagger UI (no auth required)

## Streaming Response Format

Streaming endpoints return NDJSON (one JSON object per line). Pass `Accept: text/event-stream` to receive Server-Sent Events instead.

```
{"event":"info","data":"Checking base image..."}
{"event":"stdout","data":"Hello World\n"}
{"event":"result","data":{"stdout":"Hello World\n","stderr":"","exit_code":0,"timeout":false}}
```

Event types: `info`, `stdout`, `stderr`, `result`, `error`

## Clients

Generated Node.js and Rust clients are available in the `/clients` folder.

## Development

### Server

```bash
cargo run -p tinirun-server
```

### Demo frontend

```bash
pnpm install
cd demo && pnpm dev
```

Set `DEMO_TINIRUN_URL` and `DEMO_TINIRUN_API_KEY` for the demo to connect to the server.
