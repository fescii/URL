api.md
platform interaction surface — verbs, not endpoints
status: living document
companion: tools.md (what encode/decode actually run), profile.md (what
train/load operate on), objects.md (what save/merge/fetch operate on),
urls.md (what share/open produce and consume), store.md (what everything
ultimately reads and writes through)


0. shape of this document

the interface is a small set of verbs against a core library, not a large
REST surface designed first. the verb-naming convention deliberately
follows established ML-platform practice (load, train, log, save, register,
predict) because that vocabulary already fits this system's shape well —
a profile is close enough to a model artifact that reusing proven naming
beats inventing new words for the same concepts. what's adopted and what's
deliberately left out of that convention is called out explicitly in
section 3, since not everything from that world applies here.

networked exposure (REST/gRPC) is a thin wrapper over this core, added
only if and when networked access is actually needed — the same layering
already used elsewhere in this stack (a core engine with IMAP/SMTP/REST/
gRPC surfaces on top). the core library API is the primary design surface;
network protocols are not designed first.


1. the verb set

    encode(url, profile_ref?) -> code
        run the tools.md stack against a url. if profile_ref is omitted,
        use the generic profile (profile.md §2). returns a code tagged
        with the profile version used (idea.md §4) — required for correct
        decode later.

    decode(code) -> url
        reverse of encode. resolves the profile tag embedded in the code,
        requires that exact profile version to be available (load, below)
        or fails cleanly rather than guessing.

    load(profile_ref) -> profile handle
        bring a profile (objects.md §1 manifest + blobs, or a locally
        cached copy) into active use — resolved from a content hash,
        pulled via fetch if not already local. does not modify the
        profile; purely makes it available for encode/decode/merge.

    train(corpus, options) -> profile artifact
        offline batch process (profile.md §2) producing a new profile's
        structural and statistical content from a corpus of urls — single-
        domain or multi-domain depending on what the corpus covers.
        produces an artifact, does not publish it — see save, below, for
        that distinction. mirrors the ZDICT-style offline-training
        precedent already established in profile.md, not a live/online
        operation.

    save(artifact) -> hash
        content-address and persist an artifact (profile or manifest) as
        an immutable object (objects.md §1, §3). this is the "register"
        step, deliberately separated from train — a trained artifact can
        be inspected, tested, or discarded before it's ever published as
        an addressable, shareable object. returns the hash that now
        identifies it everywhere else in this api.

    merge(refs: [...]) -> profile artifact
        combine multiple profiles per the CRDT rules in objects.md §2 —
        hash-set union for structural/literal entries, additive merge for
        frequency sketches. deterministic regardless of ref order, per
        the underlying G-Set/G-Counter guarantee. produces a new artifact;
        does not implicitly save it.

    fetch(hash) -> bytes
        retrieve an object by content hash from wherever it's reachable
        (local store, a peer, wherever this deployment is configured to
        look) — the git-fetch-equivalent operation. verifies the returned
        bytes hash to the requested key before returning (never trust an
        unverified fetch result).

    export(refs: [...], profile_ref) -> .urls file
        package a set of link entries plus a pinned profile reference into
        a portable single file (urls.md). the profile_ref becomes that
        file's prerequisite field — required, not inferred.

    open(file) -> [urls, liveness-snapshot?]
        the .urls-file counterpart to decode — verifies integrity
        (urls.md §4), resolves the pinned profile (via load, fetching if
        needed), decodes every entry. this is the operation a CLI opener
        or OS file-association handler ultimately calls (urls.md §6).

    stat(shortcode) -> { frequency, liveness state }
        read-only lookup against the store (profile.md §3 frequency
        signal, profile.md §5 liveness state). never triggers a liveness
        check itself — see health-check, below, for that.

    health-check(shortcode | all) -> updated liveness state
        explicitly trigger the background liveness worker (profile.md §5)
        for one shortcode or a batch. separated from stat on purpose:
        reading current state must stay fast and side-effect-free; forcing
        a check is a distinct, explicitly-invoked operation, never
        implicit on a decode or stat call.

    verify(object | file) -> bool
        recompute and check a content hash (an object's own identity, or
        a .urls file's integrity field) without decoding or trusting the
        contents further. the same operation underlies save's internal
        consistency checks and open's first step.


2. what each verb does NOT do (kept explicit, matches project's scope
   discipline)

  - load never mutates a profile — profiles are immutable once saved
    (objects.md §3). there is no update(profile) verb; there is only
    train producing a new artifact and save publishing a new version.
  - train never publishes automatically — save is a separate, deliberate
    step, so a trained artifact can be evaluated before it becomes an
    addressable object other calls can reference.
  - stat never performs a network check — that's health-check's job,
    kept separate so read paths stay fast (store.md's whole design
    priority) regardless of network conditions.
  - export never embeds live liveness data — only a frozen snapshot, per
    urls.md §5's explicit boundary between a static shared file and a
    live store.


3. what was deliberately NOT adopted from the ML-platform convention

  - staging/production stage transitions: MLflow-style environments
    (a model moving through staging -> production) don't map cleanly here
    — profiles are versioned and immutable, not promoted through
    environments. a caller picks a specific profile hash (or "generic" as
    a default), full stop. adding stage semantics would be state this
    project doesn't need (idea.md §3's discipline against unneeded
    generality applies directly here).
  - a tracking/experiment UI layer: ML-platform registries typically pair
    with an experiment-tracking UI (metrics, run comparisons, dashboards).
    this project isn't a research platform — train produces one artifact
    from one corpus, evaluated externally if at all. no UI layer is
    implied by this document.
  - predict/score naming: kept encode/decode instead — they describe
    what actually happens (compress and reverse a URL) more precisely
    than a generic ML-inference verb would, and avoids implying this is
    a prediction/classification system.


4. surface exposure, in order of priority

  1. core library — the verb set above, called in-process. this is the
     primary, load-bearing interface.
  2. CLI — thin wrapper over the core library, the primary human-facing
     surface (urls.md §6's "canonical opener" is this).
  3. REST/gRPC — thin network wrapper, built only when a real networked
     use case exists, following the same layering already used elsewhere
     in this stack (a core engine exposed via IMAP/SMTP/REST/gRPC). not
     designed ahead of an actual need.


5. open items

  - exact corpus input format for train (raw url list? weighted by
    observed frequency already? — affects whether train needs its own
    frequency-counting pass or expects one done already).
  - whether fetch needs a defined peer-discovery mechanism or always
    assumes an explicitly configured source (a specific store, a specific
    peer) — leaning toward the latter for v1, no discovery layer, per
    idea.md's explicit exclusion of any discovery/social layer.
  - batch semantics for health-check("all") at real scale — likely needs
    to respect the backoff schedule noted as open in profile.md §5/§6
    rather than checking everything on a fixed interval regardless of
    state.
