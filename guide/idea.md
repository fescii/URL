idea.md
zero-storage URL compressor — vision, scope, constraints
status: living document
companions: tools.md (algorithms), store.md (storage engine), objects.md (git-like
sharing), profile.md (metadata, stats, url state), url-protocol-atlas.md (research
corpus of protocols/patterns feeding the dictionary and model)


1. what this is

a URL shortener with no per-URL database. every encode/decode is computation
against small, fixed, versioned artifacts (profiles) — never a lookup into a
growing table of "shortcode -> original url" rows. the thing that grows over
time is metadata (dictionaries, frequency stats, model weights), not a record
per shortened link.

works for arbitrary third-party URLs, not just URLs from owned systems — this
was an explicit scope decision (see 3. below).


2. the mathematical floor — read this before anything else

a lossless, deterministic, storage-free function cannot guarantee a shorter
output for every possible input. this is a counting argument (pigeonhole
principle formalized as the incompressibility theorem), not an engineering
limitation, and no algorithm, however clever, changes it. for any length n
there exist strings whose shortest possible representation is not shorter
than n, and this holds against every function that could ever be written,
including ones that don't exist yet.

what IS achievable: shorter on average across real-world URL traffic,
always correctly reversible, occasionally not shorter for adversarial or
already-high-entropy input (e.g. a bare UUIDv4 token). that is the honest
product claim. "always shorter for any input" is not a target — it is
mathematically impossible and should never be promised.

the entire design effort goes into getting as close to the entropy floor as
possible for the *actual* distribution of real-world URLs, using only fixed,
non-growing computation. everything below is in service of that.


3. scope decisions already made (do not re-litigate without reason)

  in scope:
    - arbitrary third-party URLs (not just owned-system URLs — tier-1
      template packing for owned systems was explored and explicitly
      dropped in favor of a general solution)
    - structural + statistical modeling of real-world URL patterns
      (url-protocol-atlas.md is the research corpus for this)
    - profiles fine-tunable per single domain or bundled multi-domain,
      swappable without touching code (see profile.md)
    - sharable, privacy-respecting profiles and sharable link-list
      artifacts, git-like in mechanism (see objects.md)
    - a from-scratch, purpose-built storage engine, not a general database
      (see store.md)
    - basic URL liveness/state tracking (dead link detection) — the one
      deliberate exception to "no per-item state," scoped tightly
      (see profile.md section 5)

  explicitly out of scope — flagged during research specifically to
  prevent drift, do not add without a real reason:
    - any discovery/search/social layer on top of shared link lists
    - heavy ML frameworks, training pipelines, or neural inference on the
      hot path (static, offline-trained, frozen weights only — see tools.md)
    - general-purpose database features: SQL, ordered range scans, ACID
      transactions beyond what content-addressing gives for free, tombstone/
      delete support for content-addressed data (immutable by construction)
    - byte-level delta chains (git-packfile-style) between shared blobs —
      whole-entry content-addressed dedup covers most of the win at a
      fraction of the complexity for this data's shape (short strings)
    - reinforcement-learning or LSTM-based cache replacement — real research
      area, wrong budget; ARC (see store.md) is the correct-sized answer
    - content-defined chunking / rolling-hash substring dedup — exact-hash
      dedup is sufficient at this entry size; revisit only if profile sizes
      become a real problem in practice


4. system shape, one level down

    encode(url):
      route through structural knowledge (url-protocol-atlas.md) to find
      dictionary matches (scheme, authority, tracking params, id-format)
        -> grammar/SLP transform collapses matched structure
        -> residual bytes go through entropy coding driven by a profile's
           static statistical model (tools.md)
        -> bijective base encoding of the resulting bitstream
        -> short keyed tag identifies which profile produced this code
           (profile.md) — required for correct decode later

    decode(code):
      un-base the bitstream, verify integrity tag, look up the named
      profile (must be the exact version used at encode time), reverse
      entropy coding, reverse grammar transform, reconstruct url

    everything static/shared lives in profiles (structural dictionary +
    statistical weights), content-addressed and versioned (objects.md).
    everything mutable and per-item (has this code been seen, is its
    target still alive) lives in the store (store.md), never in a profile.


5. why "computation, not storage" still needs a store

worth being honest about this tension up front rather than glossing over
it: a pure zero-storage system cannot track whether a shortened link's
target is still alive, cannot serve a shared profile without something
holding its bytes, and cannot merge community-contributed dictionaries
without somewhere to merge them into. "no storage" was never literally
"no bytes persisted anywhere" — it means no per-URL database growing
linearly with usage, where the growth is proportional to *information
the system chose to remember about one specific URL* rather than *general
knowledge about URL structure*. profiles and the object store hold the
latter. the store (store.md) holds the former, deliberately minimized to
exactly liveness state and nothing else — see profile.md section 5 for
the explicit boundary.


6. top-level risks

  - entropy floor (section 2): must be communicated honestly in any
    product surface, never oversold.
  - profile version drift: a code decoded under the wrong profile version
    is silently wrong, not an error. versioning/pinning discipline in
    objects.md and profile.md is load-bearing, not optional polish.
  - scope creep toward a general platform: every one of the "explicitly
    out of scope" items above was cut for a specific, researched reason.
    reintroducing any of them should require restating that reason and
    showing it no longer holds, not just convenience in the moment.
