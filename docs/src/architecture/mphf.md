# Architecture: Minimal Perfect Hash Indexing (MPHF)

Standard hash tables (like `std::collections::HashMap` or `HashSet`) require 40 to 80 bytes of RAM per entry due to bucket arrays, open addressing slack, and raw key storage.

`urls` implements a two-level Minimal Perfect Hash Function index requiring only **3.02 to 3.62 Bytes per key**.

---

## Two-Level Pilot Seed Search

```
 [ Input Key Hash (64-bit) ]
             │
             ├───> Level 1: Bucket Mapping (Bucket ID = Hash % NumBuckets)
             │
             └───> Level 2: Pilot Hashing (Slot = (Hash ^ SplitMix64(Pilot)) % NumSlots)
                                   │
                                   v
                      [ Elias-Fano Monotone Offsets ]
```

1. **Bucket Partitioning**: Keys are partitioned into small buckets of size 3–5.
2. **Pilot Generation**: For each bucket, the builder searches for a 16-bit pilot seed such that all keys map to vacant slots without collisions.
3. **Pilot Mixing**: Pilot seeds are combined using 64-bit invertible SplitMix64 permutations, ensuring pilots retain maximum entropy.
4. **Compact Storage**: Only the 16-bit pilot seeds and 8-bit fingerprint bytes are stored in RAM.
