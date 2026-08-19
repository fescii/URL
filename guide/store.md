store.md
custom storage engine — purpose-built, not a general database
status: living document
companion: idea.md (why a store exists at all despite "no storage"),
objects.md (the content-addressed data this store holds), profile.md
(the metadata/stats this store also holds), tools.md (BLAKE3, the hash
producing this store's keys)


0. what this store is for, precisely

three, and only three, kinds of data live here:
  1. profile artifacts (dictionaries, trained weights) — objects.md/profile.md
  2. shared link-list objects (blobs + manifests) — objects.md
  3. per-shortcode mutable state (liveness/dead-link tracking) — profile.md §5

nothing else. this is not a general key-value database and should never
grow into one — see idea.md §3 for the explicit list of database features
cut on purpose. size budget: 1-3MB of code. that budget is generous for
what's actually needed here; resist filling it with generality.


1. why "faster than Redis" is mostly free, architecturally

worth stating plainly so effort goes to the right place: Redis's latency
floor is set by being a networked server, not by its data structures — an
embedded, in-process library has no network round-trip and no protocol
serialization cost at all. an embedded store already wins most of that gap
before a single algorithm decision is made. the algorithm choices below
are about what's achievable *within* the embedded-store category, which
is a smaller and more specific gain than the store-vs-Redis gap itself.


2. core structure — content-addressed hash index over an append-only log

keys throughout this system are BLAKE3 hashes (tools.md §6) — uniformly
distributed, no meaningful order, never range-scanned. that single fact
rules out B-trees and LSM-trees, both of which exist to maintain sorted
order for range queries this workload never performs. what's needed is
always an exact point lookup: "give me the bytes for this exact hash."

chosen design: Bitcask-derived — an append-only log file for durability,
plus an in-memory hash index (key -> {file, offset, length, tier}) for
O(1) average-case lookup, one hash computation plus one seek. writes are
pure sequential appends, the fastest operation any storage medium
supports. the entire engine is two data structures. this is a few hundred
KB of logic, not a database.

deliberate simplification vs classic Bitcask: no tombstone/delete
machinery. everything this store holds is either content-addressed and
immutable (profiles, objects — same bytes always hash to the same key, so
there is never an "update," only a new entry) or the narrow mutable
exception (liveness state, profile.md §5, which overwrites in place rather
than needing log-structured delete semantics). that removes a real chunk
of Bitcask's complexity for free, because the data model doesn't need it.


3. the store owns placement metadata, not just bytes

per the original requirement: the store is responsible for knowing how to
load, write, and store each entry — that's the {file, offset, length,
tier} record in the hash index, not something callers compute themselves.
callers ask for a key; the store decides which tier it's in, whether it's
mmap-resident or needs a page fault, and serves it. this metadata is
small (a fixed-size record per key) and lives entirely in memory during
normal operation, rebuilt from the log on startup if needed.


4. zero-copy read path — on-disk bytes ARE the in-memory structure

the standard read path (read bytes -> deserialize -> use) has a
deserialize step that's pure overhead. avoiding it is most of where a
store's real speed comes from: mmap-backed stores get their speed exactly
this way — the on-disk representation equals the in-memory representation,
so a "load" is a page-fault-served pointer, not a parse. this is the same
zero-copy discipline already used elsewhere in this codebase (rkyv as
substrate, serde only at boundaries) — applied here one layer down, to
storage instead of in-process structures. entries should be laid out so
that mapping them directly yields a usable struct, no deserialize call
required on the hot path.


5. chunking — page-aligned, not one giant file

rejecting monolithic blob files was correct. chunks/pages should align
with OS page size so that a "load" is a page fault the OS's own cache
machinery already knows how to serve, rather than an arbitrary-offset read
requiring hand-rolled caching logic. this is the same discipline already
applied in the hot/warm/cold tiering elsewhere in this codebase — treat it
as the same mechanism reapplied here, not a parallel system to maintain.


6. concurrency — shard-per-core, not a shared lock

the modern, proven-at-scale pattern for this is share-nothing,
shard-per-core rather than one structure behind a shared lock: partition
the keyspace by a few bits of the hash key (already uniformly
distributed, so the partition is free and balanced by construction), give
each core its own log file, its own hash index, its own hot-tier cache.
cross-core communication is explicit message passing only, never shared
memory — this is the architecture underneath ScyllaDB (already in use
elsewhere in this stack), proven at extreme throughput specifically
because it eliminates lock contention and cache-line bouncing rather than
trying to make locking cheaper. a lighter-weight fallback (a sharded
concurrent map, same principle as the DashMap pattern already used in
Ngome) is acceptable if full core-pinning is more machinery than this
specific store needs.


7. async I/O — plain mmap first, io_uring as a deliberate phase two

io_uring is the right eventual target for this shape of workload
(high-rate random point lookups against local storage), but it is not a
drop-in win: naive replacement of existing I/O calls with io_uring, with
no other architectural change, yields only marginal improvement in
practice — the real benefit requires registered buffers and deliberate
batching, engineered on purpose rather than dropped in. there is also a
real counter-case: on some workload shapes, plain epoll has outperformed
early io_uring implementations. build the synchronous mmap path first —
per section 4, that's already fast and already beats Redis
architecturally — and treat io_uring as a targeted, benchmarked
optimization once there's a real workload to measure against, not a
foundational day-one dependency.


8. adaptive hot-tier caching — ARC, not a learned/RL cache

"loads based on ML" is answered correctly at a much smaller size than a
trained model implies. real ML-based cache replacement (LSTM sequence
models, reinforcement learning) is an active research area, but it needs
training pipelines and model inference on the hot path — wrong budget
entirely for this store. the right-sized answer is ARC (Adaptive
Replacement Cache): self-tuning between recency and frequency online, no
training data, no configuration, constant-time overhead per request,
negligible space overhead, and battle-tested at production scale (this is
ZFS's cache algorithm, and IBM storage controllers use it too). ARC's
internal feedback mechanism — two "ghost" lists tracking recently evicted
entries, used to shift the recency/frequency balance automatically when a
workload changes — is the actual adaptive/"learning" behavior, achieved
with no model file and no training step.

this composes for free with profile.md's frequency sketches: the same
count-min-sketch-based "what's accessed often" signal drives both
ANS/dictionary weighting (tools.md) and ARC's hot-tier decisions here.
one frequency-tracking substrate, two consumers — do not build a second,
separate mechanism for hotness tracking in this store.


9. explicitly out of scope for this store (see idea.md §3 for the full
   project-wide list; store-specific items repeated here for locality)

  - B-trees, LSM-trees, or any structure maintaining sorted order —
    nothing here is ever range-scanned by key.
  - tombstone/delete support — content-addressed data is immutable by
    construction; the one mutable exception (profile.md §5) overwrites
    in place, it doesn't need log-structured deletion.
  - learned/RL cache replacement — ARC is the correct-sized answer.
  - io_uring on day one — phase two, after benchmarking against the
    plain mmap path.
  - SQL, transactions beyond what content-addressing gives for free,
    any query capability beyond exact-key lookup.


10. size budget sanity check

Bitcask-style engine: a few hundred KB of core logic. ARC: documented as
negligible overhead, a few hundred more lines. shard-per-core wiring:
mostly structural, not algorithmically heavy. mmap zero-copy layout:
design discipline more than code volume. 1-3MB is comfortable headroom,
not a tight target — pressure will come from adding unneeded generality,
not from these algorithms themselves.
