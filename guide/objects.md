objects.md
git-like content-addressed object model — sharable profiles and link lists
status: living document
companion: idea.md (why sharability matters here), profile.md (the specific
content this model carries for profiles), store.md (where object bytes
physically live), tools.md (BLAKE3, the hash this whole model is built on)


0. two distinct sharable things — keep them separate

  - profile objects: reusable structural/statistical knowledge about how
    URLs are shaped (profile.md). many people can use the same profile.
  - link-list objects: one person's specific collection of links they
    want to publish, version, or hand to someone else. not reusable
    knowledge — a personal artifact.

both use the same underlying mechanism (this document); they are not the
same *kind* of object and shouldn't be merged conceptually just because
the machinery is shared.


1. the core mechanism — content addressing

every object's identity is the BLAKE3 hash of its own bytes. identical
content always produces the identical key, which is what makes
deduplication free rather than a separate feature: git's own object store
works exactly this way — the name of every object is the hash of its
contents, so identical content is stored exactly once, automatically, with
no dedup pass required. this project borrows that idea directly, not the
rest of git's tooling.

two object shapes, deliberately minimal (git has four object types; this
needs two):
  - blob: raw content-addressed bytes. a dictionary entry, a trained
    weight block, a single shared link.
  - manifest: a small object listing other objects by hash — the
    equivalent of a git tree. "this profile version is these N blob
    hashes." "this link list is these M blob hashes, in this order."

both are immutable once written. there is no edit operation, only
publishing a new manifest that references a mix of old and new blobs.


2. merging without a coordinator — grow-only set semantics

when two profiles or two link lists merge (a community-contributed
dictionary combined with a personal one, two people's link lists
combined), the result must never contain duplicates, regardless of merge
order, and merging must never require a central authority to arbitrate.
this is a solved problem: a CRDT grow-only set (G-Set) has exactly this
property by construction — merging is set union, which is safe because a
set is a lattice and the only operation, add, is an idempotent inflation.
idempotent + commutative + associative means merge order never matters,
proven, not just observed in practice. this has direct precedent applied
to model-merging specifically, which is close to this exact use case.

practically: since every entry is already content-addressed (section 1),
"merge" reduces to a hash-set union — identical bytes collapse to one
entry automatically, no separate dedup logic needed. this is the same
property that makes git's own storage deduplicate for free.

numeric data (frequency counts, statistical weights — see profile.md)
merges differently and just as safely: these are linear sketches, and
linear sketches are natively mergeable — the sketch of a concatenated
stream equals the sum of the individual sketches. so numeric metadata
merges by addition (itself a standard CRDT, the G-Counter), while literal
string/blob entries merge by hash-set union (G-Set). two merge rules, both
provably conflict-free, no coordination required for either.

deliberately not used: OR-Set semantics (add/remove with tombstones).
this data model never deletes — only supersedes via a new manifest
version — so the tombstone machinery a full OR-Set needs is unnecessary
complexity here, and it's exactly the kind of complexity that has caused
real, documented surprises (state appearing to "roll back") in production
CRDT systems that carry more machinery than their data model requires.


3. versioning discipline — this is load-bearing, not optional

a code encoded against profile version N is garbage if decoded against
version N+1 with different weights. every published profile or manifest
is therefore treated as immutable and append-only: never edit a published
object, only publish a new one and reference it explicitly. every code
produced by this system carries a short tag identifying which profile
version produced it (profile.md), so a decoder can either find the exact
matching version or fail cleanly — silent misdecoding under a mismatched
profile is the one failure mode this system cannot tolerate anywhere.

for shared link-list manifests specifically: if the list references
compressed codes (not raw URLs), the manifest must pin which profile
hash those codes depend on — the same discipline as a lockfile pinning a
dependency version. omitting this is not a minor gap; it silently turns a
shared list into undecodable garbage on any profile mismatch.


4. privacy boundary on shared profiles — what's public, what's protected

not everything in a profile is equally sensitive, and conflating the two
would either over-restrict harmless data or under-protect real signal:
  - structural content (scheme templates, tracking-param names, domain
    strings, id-format grammars — url-protocol-atlas.md) is public-domain
    knowledge about how URLs are shaped. shipped in the clear, no privacy
    mechanism needed or wanted here.
  - statistical content (how often *this specific corpus* saw a given
    pattern) can leak real signal about someone's actual traffic. this is
    where protection belongs.

count-min sketches (profile.md) — the structure used for the statistical
layer — are naturally suited to formal privacy protection here: adding a
small amount of noise at initialization gives a differential privacy
guarantee while leaving the update and query algorithms completely
unchanged, at effectively zero runtime cost, and the guarantee holds no
matter how many times the sketch is queried afterward. apply this noise
layer only to statistical content, never to structural content that was
never private to begin with — don't over-engineer privacy onto public
data.


5. sharing link lists specifically

a manifest listing blob hashes plus a pinned profile-version reference
(section 3). sharing a list costs one small manifest plus whichever blobs
the recipient doesn't already have — most of a real person's link list
will share domains, tracking-param structure, and templates across
entries, so content-addressed dedup captures most of the real-world
redundancy before any cleverer technique is needed. adding one link to a
shared list costs one new blob and one new manifest; nothing is rewritten,
matching the immutability rule in section 3.

structurally this is as safe to share as any file — public URLs,
content-addressed, same trust model as sharing any other file. no
discovery, search, or social layer is included here (see idea.md §3) —
this stays a mechanical, content-addressed distribution primitive, not a
platform.


6. hashing choice

BLAKE3 uniformly (tools.md §6) — 4-10x faster than SHA-256 on typical
hardware, and its native Merkle-tree internal structure is a direct fit
for this document's Merkle-DAG-shaped object model (manifests referencing
blob hashes is already a small DAG). no part of this system needs SHA-256
specifically; if an external integration ever requires it, that's scoped
to the integration point, not adopted here.


7. explicitly deferred / cut (see idea.md §3 for full project-wide list)

  - content-defined chunking (rolling-hash substring dedup) for partial
    overlap between large entries — exact whole-entry hash dedup already
    captures nearly all the win at this entry size (short strings, tens
    of bytes); CDC's value shows up on much larger objects. revisit only
    if profile sizes become a measured problem.
  - byte-level delta chains between blob versions (git-packfile-style) —
    content-addressed dedup already removes exact-duplicate redundancy;
    byte-level diffing is git's answer to versioned *files*, not a good
    fit for many short, mostly-distinct strings. optional v2 item.
  - OR-Set tombstone/remove semantics — not needed, see section 2.
