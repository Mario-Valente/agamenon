# Agamenon - Lightweight Schema Registry

A high-performance, PostgreSQL-backed Schema Registry implementation written in Rust, compatible with the Confluent Schema Registry REST API.

## Features

✅ **REST API Compatible** - Drop-in replacement for Confluent Schema Registry endpoints
✅ **Avro Support** - Full Avro schema validation and compatibility checking
✅ **Compatibility Modes** - BACKWARD, FORWARD, FULL, NONE compatibility checks
✅ **LRU Caching** - High-speed in-memory caching with Moka (1M+ entries)
✅ **PostgreSQL Backed** - Durable persistence with ACID transactions
✅ **Basic Auth** - Simple username/password authentication
✅ **Lightweight** - No Kafka dependency required
✅ **Production Ready** - Built with Axum, Tokio, SQLx

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/subjects` | List all subjects |
| GET | `/subjects/:name/versions` | List versions for a subject |
| POST | `/subjects/:name/versions` | Register new schema |
| GET | `/schemas/ids/:id` | Get schema by global ID |
| POST | `/compatibility/subjects/:name/versions/:version` | Check compatibility |

## Setup

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+

### Installation

```bash
git clone <repo>
cd agamenon
```

### Environment Variables

```bash
DATABASE_URL=postgresql://user:password@localhost/agamenon
SERVER_HOST=127.0.0.1
SERVER_PORT=8081
LOG_LEVEL=info
SCHEMA_REGISTRY_USER=admin
SCHEMA_REGISTRY_PASSWORD=admin
CACHE_MAX_CAPACITY=1000000
```

### Running

```bash
# Build
cargo build --release

# Run (will auto-migrate database)
./target/release/agamenon
```

## Usage Examples

### Register a Schema

```bash
curl -X POST http://localhost:8081/subjects/user-value/versions \
  -H "Authorization: Basic $(echo -n 'admin:admin' | base64)" \
  -H "Content-Type: application/json" \
  -d '{
    "schema": "{\"type\":\"record\",\"name\":\"User\",\"fields\":[{\"name\":\"id\",\"type\":\"int\"},{\"name\":\"name\",\"type\":\"string\"}]}",
    "schema_type": "AVRO"
  }'
```

### Get Schema by ID

```bash
curl http://localhost:8081/schemas/ids/1 \
  -H "Authorization: Basic $(echo -n 'admin:admin' | base64)"
```

### Check Compatibility

```bash
curl -X POST http://localhost:8081/compatibility/subjects/user-value/versions/1 \
  -H "Authorization: Basic $(echo -n 'admin:admin' | base64)" \
  -H "Content-Type: application/json" \
  -d '{
    "schema": "{\"type\":\"record\",\"name\":\"User\",\"fields\":[{\"name\":\"id\",\"type\":\"int\"},{\"name\":\"name\",\"type\":\"string\"},{\"name\":\"email\",\"type\":[\"null\",\"string\"],\"default\":null}]}"
  }'
```

## Testing

```bash
cargo test
```

## Architecture

- **routes/**: HTTP handlers for REST endpoints
- **models/**: Domain models (Schema, Subject, CompatibilityLevel)
- **services/**: Business logic (CompatibilityChecker)
- **storage/**: Database abstraction (SchemaStore trait + PostgreSQL impl)
- **cache/**: Moka LRU cache wrapper
- **auth/**: Basic HTTP authentication
- **config/**: Configuration from environment

## Performance

- **Reads**: Sub-millisecond with LRU cache
- **Writes**: ~10-50ms depending on database
- **Memory**: ~100 bytes per cached schema

## License

MIT
