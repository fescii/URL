urls.md
the .urls file format — a portable, single-file, shareable artifact
status: living document
companion: objects.md (the abstract manifest/blob model this format
serializes into one portable file), profile.md (pinned profile dependency,
liveness state boundary), idea.md (scope discipline this format respects)


0. what this is, and how it differs from objects.md

objects.md defines the abstract model: content-addressed blobs, manifests,
CRDT merge rules. that model assumes access to a store (store.md) — blobs
can live wherever, referenced by hash. a .urls file is the concrete answer
to a narrower, more human question: "I want to hand you one file, over
email, a chat app, a USB stick, and you should be able to open it without
needing access to anything else." this document is a serialization format,
not a new data model — it packages objects.md's manifest + referenced
blobs into a single, self-contained, portable container.


1. structural precedent — git bundle, not a generic archive

git already solved almost exactly this problem: packaging a subset of a
content-addressed object graph into one transferable file, self-describing
enough that the receiving end knows what it has and what it's missing. its
bundle format is the direct structural template here — a bundle is a
plain-text header (signature line, prerequisites, references) followed by
packed object data. two ideas from that header are load-bearing for
.urls specifically:

  - prerequisites: git bundle's header lists objects the bundle does NOT
    contain but requires the receiving side to already have, so an
    incremental bundle can be small. the equivalent here is the pinned
    profile-version hash (objects.md §3, profile.md §2) — a .urls file
    does not need to embed the profile it depends on, but it MUST
    declare which profile hash it was encoded against, or decoding is
    silently wrong rather than cleanly impossible.
  - self-contained vs exclusion-based bundles: git supports both a fully
    self-contained bundle (extractable anywhere, no prior history needed)
    and a thin bundle (smaller, but requires the recipient to already
    hold certain objects). .urls should default to self-contained
    (embed the manifest and all referenced link blobs directly) since the
    whole point is "hand someone one file that just works" — a thin/
    exclusion-based variant is a reasonable later optimization for very
    large lists shared between parties who already share most objects,
    not a v1 requirement.


2. file structure

    [ magic ]        fixed byte sequence identifying the format
                      (section 3) — always first, always fixed length
    [ version ]       single byte, format version
    [ prerequisite ]  pinned profile hash (BLAKE3, tools.md §6) — the
                      profile this file's codes were encoded against.
                      required, not optional (objects.md §3)
    [ manifest ]      list of entry references — see objects.md §1's
                      manifest object, serialized inline here rather
                      than referenced externally
    [ blobs ]         the referenced link entries themselves — each one
                      an encoded code (tools.md output) plus whatever
                      per-entry fields the manifest declares (see
                      section 5 on liveness snapshots)
    [ integrity ]     BLAKE3 hash of everything preceding this field,
                      appended last — a decoder verifies this before
                      trusting anything else in the file, same discipline
                      as file signature verification generally: a magic
                      number identifies the *type*, a content hash
                      verifies the *contents* haven't been altered or
                      truncated, and these are deliberately separate
                      concerns.

everything from magic through the integrity field is plain, inspectable
structure — no encryption, no obfuscation. a .urls file is meant to be
shared; there's nothing here that should require a secret to parse.


3. magic bytes / file signature — identifiable independent of extension

file type identification tools (the unix file command and equivalents)
work primarily off a fixed byte sequence at a known offset, not the
extension — extensions are a convenience, not a reliable identifier, since
nothing stops a file from being renamed. a .urls file should carry its own
fixed magic sequence at offset 0 so it is identifiable regardless of what
it's named or where it's found (recovered from a disk image, attached to
an email with the extension stripped, etc). standard practice is a short
distinctive byte sequence chosen to be unlikely to collide with existing
registered formats — pick this deliberately and register it in whatever
local `file`/magic database tooling is in use, the same way any other
format shows up in that database.


4. integrity, not encryption

the appended BLAKE3 hash (section 2) protects against corruption and
truncation — a decoder that computes a mismatched hash should refuse to
proceed rather than attempt a partial decode. this is a correctness
mechanism, not a security boundary; nothing about a .urls file assumes
the contents are secret, since by design (objects.md §5) this is meant to
be freely shareable. if a future need arises for a private/access-
controlled variant, that's a different, explicitly-scoped extension, not
a default behavior.


5. what decoding reveals

a decoder that has the pinned profile (section 2's prerequisite field)
can reverse every entry back to its original URL. what's included per
entry beyond the code itself is deliberately minimal:

  - the encoded code (required)
  - optionally, a liveness snapshot at export time (alive/dead/unknown —
    profile.md §5's state values) — explicitly a snapshot, frozen at the
    moment the file was created, not a live connection to the exporting
    party's store. a .urls file is static once written; it does not
    reach back into anyone's store.md instance to fetch current state.
    a decoder should treat any included liveness field as "as of export,"
    never as current truth, and this should be visible to whoever's
    reading the decoded output, not silently presented as live data.

nothing else. no analytics, no tracking metadata beyond what the original
URLs themselves may have carried (url-protocol-atlas.md §4.1) — a shared
list is exactly the URLs someone chose to include, decoded faithfully.


6. opening it — CLI first, OS association as an optional layer on top

the canonical way to open a .urls file is a small dedicated tool that
verifies the integrity field, resolves the prerequisite profile, and
decodes the manifest — the same relationship a .torrent file has to a
torrent client, or a git bundle has to `git bundle verify` / `git clone`
on the bundle path. this keeps the format itself dependency-free and
matches the project's CLI-first, self-hosted-infrastructure disposition
(idea.md) rather than requiring a GUI to be meaningful.

double-click / OS-level association is a legitimate nice-to-have on top of
that CLI tool, not a replacement for it, and the mechanism differs by
platform:

  - linux: register a MIME type (following the convention of an
    unregistered/vendor type, e.g. application/x-<name>) via an XML
    file dropped in the shared-mime-info system's packages directory,
    declaring the magic bytes (section 3) and the *.urls glob pattern,
    paired with a .desktop entry naming the CLI tool (or a thin GUI
    wrapper around it) as the handler. this is the standard freedesktop.org
    mechanism used across GNOME/KDE and any XDG-compliant environment.
  - macos: declare a Uniform Type Identifier for the format (exported
    type, associated extension and magic-byte signature) in the handling
    application's bundle metadata, so Finder and other apps recognize the
    type by content, not just extension.
  - windows: register a ProgID associated with the .urls extension in the
    registry, pointing at the handling executable.

none of this OS-registration work is required for the format to function
— it's presentation/convenience layered on a CLI tool that works
identically with or without it. keep it explicitly optional and low
priority relative to the CLI and the format itself (idea.md's scope
discipline applies here too: a polished cross-platform installer/GUI
experience is a different, much larger project than a working file
format).


7. relationship to objects.md, restated

.urls is not a competing design to the manifest/blob model in objects.md
— it's that model, serialized into one portable container instead of
living across a store. anything that changes in objects.md's merge or
versioning rules (section 2, section 3 there) applies here unchanged; this
document only adds the container framing (magic bytes, integrity field,
prerequisite declaration, OS-opening behavior) needed to make that model
handoff-able as a single file.


8. open items

  - exact magic byte sequence and format version numbering scheme — pick
    once, don't bikeshed; low cost to get slightly wrong early since
    version bumps are expected.
  - whether a thin/exclusion-based variant (section 1) is worth building
    before there's a real case of very large lists shared between parties
    who mostly already share objects — defer until that case is real.
  - whether the OS-association layer (section 6) is worth building at all
    versus staying CLI-only indefinitely — genuinely open, revisit once
    there's a concrete user need for double-click behavior specifically.
