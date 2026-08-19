# Architecture: Zero-Copy Memory-Mapped Engine

High-throughput point lookups depend on eliminating memory allocations from the critical path.

---

## Memory-Mapped Bitcask Logs

Traditional storage engines allocate a new heap buffer (`Vec<u8>`) on every single file read. `urls` utilizes memory-mapped files (`memmap2::Mmap`) paired with the `bytes::Bytes` abstraction.

```
 [ Disk File ] ───> [ OS Page Cache / Mmap ] ───> [ Bytes Slice (Pointer + Len) ]
                                                           │
                                                           v
                                              [ Returned to Caller / Network ]
                                              (Zero Heap Allocations on Read)
```

---

## ARC Cache Integration

The Adaptive Replacement Cache (`Cache`) stores `bytes::Bytes` handles directly. 

When a point lookup hits the cache:
- The `Bytes` reference count is incremented (an atomic operation).
- **No data copying occurs**.
- The same underlying memory buffer is shared across thread workers and network responses.
