# `urls encode` & `urls decode`

Stateless algorithmic compression and decompression. These commands operate entirely mathematically with zero network calls and zero database state.

---

## `urls encode`

Compresses a raw URL into an algorithmic shortcode prefixed with an encoding tag (`0`, `1`, `2`, or `3`).

### Syntax
```bash
urls encode <URL> [--profile <PATH>]
```

### Options
- `<URL>`: The raw target URL string to encode.
- `-p, --profile <PATH>` *(optional)*: Path to a pre-trained frequency profile for enhanced Asymmetric Numeral Systems (rANS) compression.

### Example
```bash
urls encode "https://github.com/rust-lang/rust/pull/12345"
```
Output:
```
3https://github.com/rust-lang/rust/pull/12345
```

---

## `urls decode`

Decodes an algorithmic shortcode back into its exact original URL.

### Syntax
```bash
urls decode <CODE> [--profile <PATH>]
```

### Options
- `<CODE>`: The encoded shortcode string.
- `-p, --profile <PATH>` *(optional)*: Matching frequency profile used during encoding.

### Example
```bash
urls decode "0CEbn5mgfiOdL~1zn6DNmt-Znr88EDGMFLXluKIXALluOM6DfX9.7f4pr55u9fQ9~..."
```
Output:
```
https://shop.google.com/user/profile/distribution
```
