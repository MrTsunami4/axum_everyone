# Axum Everyone

A lightweight REST API for managing jokes, built with Rust and the Axum web framework.

## Features

- Full CRUD operations (Create, Read, Update, Delete)
- Pagination support for listing jokes
- Random joke endpoint
- Health check endpoint
- SQLite database with automatic migrations
- Input validation
- CORS support
- Graceful shutdown handling
- Comprehensive test coverage

## Tech Stack

- **Runtime**: [Tokio](https://tokio.rs/) - Async runtime
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) v0.8
- **Database**: SQLite via [SQLx](https://github.com/launchbadge/sqlx) (compile-time checked queries)
- **Serialization**: [Serde](https://serde.rs/)
- **Validation**: [validator](https://github.com/Keats/validator)
- **CLI Parsing**: [Clap](https://clap.rs/)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) + [tower-http](https://github.com/tower-rs/tower-http)

## Quick Start

### Prerequisites

- Rust (edition 2024, i.e., Rust 1.85+)
- Cargo

### Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd axum_everyone
   ```

2. Configure environment variables:
   ```bash
   cp .env.example .env
   ```
   
   The default `.env` file contains:
   ```
   DATABASE_URL=sqlite:data.db
   ```

### Build and Run

```bash
# Build the project
cargo build

# Run the server (starts on http://localhost:3000)
cargo run
```

### CLI Options

```bash
cargo run -- --help
```

Available options:
- `--host` - Bind to all interfaces (default: localhost only)
- `--port <PORT>` - Port to listen on (default: 3000)

Examples:
```bash
# Run on default (localhost:3000)
cargo run

# Run on all interfaces, port 8080
cargo run -- --host --port 8080
```

## API Endpoints

| Method | Path              | Description             | Status Codes           |
|--------|-------------------|-------------------------|------------------------|
| GET    | `/`               | Hello World             | 200                    |
| GET    | `/health`         | Health check            | 200                    |
| GET    | `/jokes`          | List jokes (paginated)  | 200                    |
| POST   | `/jokes`          | Create a new joke       | 201, 422               |
| DELETE | `/jokes`          | Delete all jokes        | 200, 500               |
| GET    | `/joke/{id}`      | Get joke by ID          | 200, 404               |
| PUT    | `/joke/{id}`      | Update joke by ID       | 200, 404, 422          |
| DELETE | `/joke/{id}`      | Delete joke by ID       | 200, 404               |
| GET    | `/joke/random`    | Get a random joke       | 200, 404               |

### Request/Response Examples

#### Create a joke (POST `/jokes`)

**Request:**
```json
{
  "content": "Why did the programmer quit his job? Because he didn't get arrays."
}
```

**Response (201 Created):**
```json
{
  "id": 1,
  "content": "Why did the programmer quit his job? Because he didn't get arrays.",
  "created_at": "2026-04-10 22:27:14",
  "updated_at": "2026-04-10 22:27:14"
}
```

#### Get jokes list (GET `/jokes`)

**Response:**
```json
{
  "jokes": [...],
  "total": 42,
  "page": 1,
  "per_page": 20,
  "total_pages": 3
}
```

#### Pagination

The jokes list endpoint supports query parameters:
- `page` - Page number (default: 1)
- `per_page` - Items per page (default: 20, max: 100)

Example: `GET /jokes?page=2&per_page=10`

#### Error Response

```json
{
  "error": "Not found"
}
```

#### Validation Error Response (422)

```json
{
  "error": "Joke content must be between 1 and 1000 characters"
}
```

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS jokes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL CHECK(length(content) > 0 AND length(content) <= 1000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Database migrations are automatically applied on application startup.

## Testing

### API Integration Tests

**PowerShell (Windows):**
```powershell
.\test_api.ps1
```

**Bash (Linux/macOS/WSL):**
```bash
./test_api.sh
```

### Rust Unit & Integration Tests

```bash
cargo test
```

8 integration tests cover all CRUD operations, validation, pagination, and error cases using isolated in-memory SQLite databases.

### Linting

```bash
cargo clippy -- -D warnings
```

## Project Structure

```
src/
├── lib.rs           # Library crate: app builder, re-exports (testable)
├── main.rs          # Binary crate: CLI, server startup, migrations
├── models.rs        # Data models and shared types
└── handler/
    ├── mod.rs       # Handler module root, error types, route handlers
    ├── create.rs    # Database insert operations
    ├── read.rs      # Database query operations (with pagination)
    ├── update.rs    # Database update operations
    └── delete.rs    # Database delete operations
migrations/
└── 001_create_jokes_table.sql  # SQLx migration
```

## CI/CD

The project includes a GitHub Actions workflow (`.github/workflows/build.yml`) that runs on every push and pull request:

1. **Build** - Compiles the project
2. **Clippy** - Runs linting (treats warnings as errors)
3. **Test** - Runs the test suite

## Development Conventions

- **Library + Binary pattern**: `lib.rs` exports `create_app()` for testable router construction; `main.rs` handles CLI and server lifecycle
- **Typed AppState**: Uses `AppState { db: pool }` for extensibility
- **Error Handling**: Custom `AppError` enum with HTTP status mapping (404, 422, 500) and structured JSON error responses
- **Validation**: Uses `validator` crate for declarative field validation
- **Database Queries**: Static SQL strings with SQLx compile-time query checking
- **Code Style**: Strict Clippy lints enabled (`all`, `pedantic`, `nursery`)
- **Documentation**: All public `Result`-returning functions have `# Errors` doc sections

## License

[Add your license here]
