# Architecture: Multi-Tier Compression Pipeline

`urls` structures compression across multiple specialized tiers, combining exact algorithmic deduction with statistical entropy and succinct indexing.

---

## The Four Compression Tiers

```
 +-------------------------------------------------------------------------+
 |                               Input URL                                 |
 +------------------------------------+------------------------------------+
                                      |
                                      v
                  +---------------------------------------+
                  |           Tier Selector               |
                  +-------------------+-------------------+
                                      |
         +----------------------------+----------------------------+
         |                                                         |
         v                                                         v
 +-------------------------------+                         +-------------------------------+
 |  Tier 1: Myers Bit-Parallel   |                         |  Tier 2: Stateful Bitcask     |
 |  Positional Delta (Stateless) |                         |  5-11 Char Base66 Code        |
 +---------------+---------------+                         +---------------+---------------+
                 |                                                         |
                 v                                                         v
 +-------------------------------+                         +-------------------------------+
 |  Tier 1.5: Structural Symbol  |                         |  Sealed Succinct MPHF Index   |
 |  Grammar & Entropy (FSST/rANS)|                         |  (3.02 - 3.62 Bytes / Key)    |
 +-------------------------------+                         +-------------------------------+
```

---

### Tier 1 — Myers Bit-Parallel Positional Delta
- Deduplicates URLs sharing structure with 32-bit MinHash centroids.
- Characters matching centroid anchors consume **0 bytes** in the substitution diff stream.
- Non-matching characters are packed into compact byte streams alongside delta bitmasks.

---

### Tier 1.5 — FSST & rANS Entropy
- Compresses structural centroid anchors using shard-level 256-symbol static tables, achieving $> 2.5\text{ GB/s}$ decompression speed.
- Re-Pair grammar compaction extracts repetitive token pairs.

---

### Tier 2 — Variable Length Sharded Store
- Computes deterministic Base66 shortcuts from 64-bit SipHash digests.
- Starts at **5 characters** (e.g. `Urh7.`, `Q4.10`) for $<50\text{K}$ items.
- Dynamically scales to 7–9 characters (`s_Urh7.DV`) upon collision detection at millions of keys.
