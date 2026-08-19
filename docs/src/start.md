# Getting Started

Get up and running with `urls` in under two minutes.

---

## 1. Installation

### From Source (Rust Cargo)

Ensure you have Rust stable (1.80+) installed:

```bash
git clone https://github.com/fescii/URL.git
cd URL
cargo build --release
```

The compiled binary will be located at `target/release/urls`. You can alias or add it to your system `PATH`:

```bash
cp target/release/urls /usr/local/bin/urls
```

---

## 2. Quickstart

### A. Squeeze a URL Without a Database (Stateless)

```bash
urls encode "https://shop.google.com/products/deals?id=123&source=newsletter"
```
Output:
```
0CEbn5mgfiOdL~1zn6DNmt-Znr88EDGMFLXluKIXALluOM6DfX9.7f4pr55u9fQ9~...
```

Expand it back without any database:
```bash
urls decode "0CEbn5mgfiOdL~1zn6DNmt-Znr88EDGMFLXluKIXALluOM6DfX9.7f4pr55u9fQ9~..."
```

---

### B. Generate a 5-Character Shortcode (Stateful)

```bash
urls shorten "https://shop.google.com/products/deals?id=123&source=newsletter"
```
Output:
```
Urh7.
```

Retrieve the original URL:
```bash
urls expand "Urh7."
```

---

### C. Launch the HTTP Redirect Server

```bash
urls serve --port 8080 --store .urls_store
```

Now test it in your browser or curl:
```bash
curl http://localhost:8080/Urh7.
# Returns HTTP 301/302 Redirect to original target URL
```
