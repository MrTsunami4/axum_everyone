# axum_everyone

A small example REST API built with [Axum](https://github.com/tokio-rs/axum) and
[Toasty](https://github.com/tokio-rs/toasty) (SQLite).

## Running

Install the stable Rust toolchain (edition 2024).

1. Copy `.env.example` to `.env` and adjust `DATABASE_URL` if needed.
2. Apply migrations and start the server:

```sh
cargo run
```

Migrations are applied automatically at startup. `DATABASE_URL` defaults to
`sqlite:./data.db`. Both executables read `.env`; existing environment variables
take precedence. `RUST_LOG` controls logging.

The server listens on `http://127.0.0.1:3000` by default. Pass `--host` to bind
to all interfaces and `--port` to change the port.

## Swagger UI

Available at `http://127.0.0.1:3000/api` once running.
The OpenAPI document is served at `/api-docs/openapi.json`.

## API

| Path | Methods |
| --- | --- |
| `/` and `/health` | GET |
| `/users` | GET, POST, DELETE |
| `/users/{id}` | GET, PUT, DELETE |
| `/users/{user_id}/jokes` | GET, POST |
| `/jokes` | GET, DELETE |
| `/jokes/{id}` | GET, PUT, DELETE |

User requests contain `name` (1–255 characters) and a valid `email`.
Joke requests contain `content` (1–1000 characters). Duplicate emails return
409; missing resources return 404. Validation failures return 400 with a JSON
`message` field.

`GET /jokes?page_size=10&cursor=42` returns `{ "items": [...], "cursor": ... }`.
Page sizes range from 1 to 100 (default 10). Pass the returned cursor to fetch
the next page; a null cursor marks the end. Deleting a user also deletes their
jokes. Multi-step writes use transactions to prevent partial changes.

## Toasty CLI

```sh
cargo run --bin cli -- migration generate
```

## Tests

```sh
cargo test
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
```

Integration tests use isolated in-memory SQLite databases and the embedded
migrations. They do not access `data.db`.
