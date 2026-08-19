# `urls shorten` & `urls expand`

Stateful URL shortening backed by the sharded Bitcask storage engine. Generates ultra-compact 5 to 11 character keys.

---

## `urls shorten`

Generates the shortest possible non-colliding Base66 shortcut (`5` to `11` characters using `0-9A-Za-z.~-_`) and stores the URL payload in the append-only sharded database.

### Syntax
```bash
urls shorten <URL> [--dir <PATH>]
```

### Options
- `<URL>`: Target URL string to shorten.
- `-d, --dir <PATH>` *(default: `.urls_store`)*: Directory path where the database is persisted.

### Example
```bash
urls shorten "https://shop.cloudflare.com/wiki/articles/latest-deals"
```
Output:
```
Q4.10
```

---

## `urls expand`

Resolves a shortcode key against the sharded storage engine using zero-copy memory-mapped reads.

### Syntax
```bash
urls expand <KEY> [--dir <PATH>]
```

### Options
- `<KEY>`: The 5–11 character shortcode key (e.g. `Q4.10` or `s_Urh7.DV`).
- `-d, --dir <PATH>` *(default: `.urls_store`)*: Database directory.

### Example
```bash
urls expand "Q4.10"
```
Output:
```
https://shop.cloudflare.com/wiki/articles/latest-deals
```
