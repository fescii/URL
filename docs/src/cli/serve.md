# `urls serve`

Launches the asynchronous, zero-allocation HTTP redirect and link resolution daemon.

---

## Usage

```bash
urls serve [OPTIONS]
```

### Options

| Flag | Long Flag | Default | Description |
|---|---|---|---|
| `-h` | `--host` | `127.0.0.1` | Network interface to bind. |
| `-p` | `--port` | `8080` | TCP port to listen on. |
| `-d` | `--store` | `.urls_store` | Path to initialized Bitcask database. |
| `-c` | `--cache` | `100000` | Adaptive Replacement Cache (ARC) capacity in entries. |
| `-t` | `--threads` | *(auto)* | Worker thread pool count. |

---

## HTTP API Endpoints

### 1. Direct Shortcode Redirect
```http
GET /:key HTTP/1.1
Host: localhost:8080
```
- **Response**: `HTTP/1.1 302 Found` with `Location: <Original_URL>`.

### 2. Stateless Code Decompression
```http
GET /0CEbn5mgfiOdL~... HTTP/1.1
Host: localhost:8080
```
- **Response**: Decompresses algorithmically on the fly and returns `HTTP/1.1 302 Found`.

### 3. Create Short URL (POST API)
```http
POST /shorten HTTP/1.1
Host: localhost:8080
Content-Type: application/json

{"url": "https://shop.google.com/products/deals?id=123"}
```
- **Response**:
```json
{
  "key": "Urh7.",
  "short_url": "http://localhost:8080/Urh7."
}
```

### 4. Health Check
```http
GET /health HTTP/1.1
Host: localhost:8080
```
- **Response**: `HTTP/1.1 200 OK` with `{"status": "ok", "uptime_sec": 142}`.
