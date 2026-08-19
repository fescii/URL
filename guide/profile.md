profile.md
metadata design — profile types, how url stats are counted, url liveness state
status: living document
companion: idea.md (the "no storage" boundary this document defines
precisely), tools.md (the statistical model profiles feed), objects.md (how
profiles are packaged, versioned, merged, shared), store.md (where the one
piece of genuinely mutable state in this document actually lives)


1. two kinds of metadata, not one — restated precisely from objects.md §4

  structural: scheme templates, tracking-param names, domain strings,
  id-format grammars. public knowledge, immutable once trained, shipped in
  the clear, no privacy concern.

  statistical: how often a pattern was actually observed. this is what
  makes a profile "fine-tuned" rather than generic, and it's the part that
  can leak real signal about someone's traffic, so it's the part that gets
  privacy treatment (section 3).

both live inside a profile object (objects.md §1, as blobs referenced by
a profile manifest). neither is per-URL storage — a profile describes a
*pattern*, shared across every URL matching that pattern, not a record
per individual link.


2. profile types

  generic: broad-corpus-trained, ships as the always-available default,
  baked into the binary. no fine-tuning applied. this is the fallback
  every encode/decode can rely on with zero configuration.

  single-domain: trained from one domain's own URL history, offline,
  batch (same process as zstd's ZDICT trainer, which is direct working
  precedent — its own documentation states plainly that dictionary
  training works when there's correlation in a family of small samples,
  and the more data-specific a dictionary is, the more efficient it is,
  concluding explicitly that there is no universal dictionary). concrete
  budget precedent from the same source: a reasonable dictionary size is
  around 100KB, trained from a corpus roughly 100x that size. useful
  starting numbers for a single-domain profile here, not hard limits.

  multi-domain: several single-domain profiles bundled under one
  manifest, selected per-entry rather than merged into one undifferentiated
  blob — a request against domain A should use domain A's trained
  statistics, not an average smeared across A, B, and C.

selection between profile types happens via the leading tag bits on each
encoded code (idea.md §4) — the tag names which profile version produced
this code, which is required for correct decode regardless of which
profile type was used.


3. how url stats are counted

the statistical layer uses linear frequency sketches (count-min-sketch
style), not exact per-string counters — this is a deliberate choice, not
a shortcut:
  - sub-linear space: memory doesn't grow with the number of distinct
    URLs observed, only with the desired accuracy/error bound. this is
    what keeps a profile's statistical section small regardless of how
    large the training corpus was.
  - natively mergeable: the sketch of a concatenated stream equals the
    sum of the individual sketches, so combining two profiles' stats
    (objects.md §2) is a simple, provably-correct addition — no special
    merge logic required.
  - naturally privacy-friendly before any explicit privacy layer is even
    added: a sketch is a many-to-one hashed structure, so it cannot be
    inverted back to "was this exact string present" even before noise is
    applied. differential privacy (section 4 below) is added on top of
    this natural property, not instead of it.

what gets counted: pattern frequency (which scheme+domain template, which
tracking-param combination, which id-format shape is common), fed to two
consumers — the entropy model's probability weighting (tools.md §5) and
the storage layer's hot-tier decisions (store.md §8). one frequency-
tracking substrate, two consumers, per the explicit non-duplication note
in store.md.


4. privacy on shared statistics

count-min-style sketches are inherently amenable to differential privacy:
adding a small amount of noise at initialization provides a formal privacy
guarantee while leaving the update and query algorithms themselves
completely unchanged — meaning this costs nothing on the runtime path,
only a one-time noise injection when a profile's sketch is built for
sharing. the guarantee holds regardless of how many times the sketch is
queried afterward (protected by post-processing immunity), so a shared
profile can safely expose real frequency statistics without letting
anyone confirm whether a specific URL was in the training corpus.

applied only to statistical content, per objects.md §4 — never to
structural content, which was never private to begin with.


5. url state — liveness / dead-link tracking (the deliberate exception)

this is the one place in the whole system that is genuinely per-item,
mutable state, and it should be named as an explicit, bounded exception
rather than quietly reintroducing "a database" — see idea.md §5 for why
this doesn't violate the project's core constraint.

what's tracked, per shortcode (or per content-hash if that's the chosen
key): a small state value, not a growing history —

    state:  unknown | alive | dead | redirect-changed | error

    unknown           — never checked, or check pending
    alive             — last check succeeded, target reachable
    dead              — last check(s) failed past a retry threshold
    redirect-changed  — target now resolves somewhere other than what
                         was recorded at encode time (only meaningful if
                         the system records a fingerprint of the original
                         target — see open question below)
    error             — check itself failed (network, timeout) — distinct
                         from dead; should not immediately count as dead

storage: this lives in the store (store.md), as an ordinary small
fixed-size record keyed by shortcode, updated in place — not
content-addressed, not versioned, not shared. this is intentionally the
one piece of data in the entire system that behaves like a conventional
mutable database row, because liveness is inherently a fact about *right
now*, not a piece of reusable structural knowledge that belongs in a
profile.

how state gets updated: background health-check workers, using the
multicore worker pool already designed in store.md §6 — each shard checks
targets for the shortcodes it owns, no cross-core coordination needed
since the state itself is sharded the same way the store shards
everything else. checks should not happen synchronously on decode — that
would add network latency to every lookup and contradicts the "absolute
speed" priority; liveness is checked periodically and out-of-band, decode
always returns the current cached state regardless of how stale it is.

backoff: a target that's currently alive should be checked infrequently;
a target that just went dead should be re-checked a few times before
being marked dead with any confidence (transient failures are common —
a single timeout is not evidence of a dead link). exact backoff schedule
is a tuning decision, not an architectural one — flag as open, not
blocking.

explicit boundary: this state is never part of a profile, never
content-addressed, never shared or merged via objects.md's mechanism.
mixing it into that system would break the immutability discipline
objects.md §3 depends on — liveness state changes constantly and has
exactly one owner (whoever operates that shortcode), which is the
opposite of a versioned, shared, multi-party artifact.


6. open questions (flag, don't block on)

  - does "redirect-changed" require storing a fingerprint (hash) of the
    originally-encoded target at encode time, to detect drift later? that
    would be a second small per-shortcode field, same storage treatment
    as state above — worth deciding once the liveness feature is actually
    being built, not before.
  - check cadence / backoff schedule (section 5) — a tuning parameter,
    revisit with real traffic data once available.
  - whether "dead" should ever cause an encode-time behavior change (e.g.
    refusing to re-serve a known-dead code) or purely an informational
    signal surfaced to whoever's asking — product decision, not covered
    by this document.
