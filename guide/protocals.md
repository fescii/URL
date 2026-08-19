url-protocol-atlas.md
research reference: protocols, schemes, and structural patterns of internet URLs
feeds: zero-storage URL compressor (bijective base + grammar transform + ANS + static model)
status: living document, append as we go
last researched: 2026-08-19


0. purpose

a compressor cannot beat entropy it does not know exists. this document is the
corpus of *structural knowledge* the compressor's dictionary, grammar rules,
and static probability model are trained against. every section below is a
candidate source of exploitable redundancy. sections are additive — extend in
place, do not reorganize without reason, keep entries terse.

cross-reference: see prior research thread for the encoding stack this atlas
feeds (bijective base-66 → grammar/SLP transform → ANS entropy coding →
optional static neural context model).


1. generic syntax — the substrate every scheme sits on

RFC 3986 (URI), RFC 3987 (IRI, unicode-aware superset).

    URI = scheme ":" ["//" authority] path ["?" query] ["#" fragment]
    authority = [userinfo "@"] host [":" port]

    scheme      ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
    unreserved  ALPHA / DIGIT / "-" / "." / "_" / "~"
    reserved    gen-delims / sub-delims
    gen-delims  ":" / "/" / "?" / "#" / "[" / "]" / "@"
    sub-delims  "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
    pct-encoded "%" HEXDIG HEXDIG

structural notes relevant to compression:
  - unreserved set (66 chars incl. the four punctuation marks) is the
    alphabet a bijective base should target — anything outside it round-trips
    through percent-encoding, which is where 3x bloat comes from on
    non-ASCII paths (IRIs). unicode-heavy paths (many Kenyan, e.g. arabic,
    or emoji-in-slug cases) should be normalized to IRI form before percent-
    decoding, THEN compressed — compressing the percent-encoded form wastes
    bits re-deriving structure ("%20" repeated is a grammar-compression
    artifact, not real entropy).
  - authority host portion follows DNS label rules (RFC 1035) separately
    from URI generic syntax — labels ≤63 octets, full name ≤253, case-
    insensitive. this bounds the dictionary token space for domains.
  - query is technically unstructured per RFC 3986 (opaque to the generic
    grammar) — the "key=val&key=val" convention is a W3C HTML convention
    (application/x-www-form-urlencoded), not a URI-level rule. this matters:
    it means query strings are the least standardized part of a URL and the
    most valuable target for statistical (not grammatical) modeling.


2. scheme registry — grouped by function

IANA maintains the canonical list under RFC 7595 (BCP 35), updated by
RFC 8615. as of the November 2025 count there were 298 registered entries,
split Permanent / Provisional / Historical; new schemes are added roughly
monthly (First Come First Served for Provisional). source of truth:
iana.org/assignments/uri-schemes — re-pull periodically, this list drifts.

registration status matters for prioritization: Permanent schemes have
gone through expert review and tend to be long-lived; Provisional (the
large majority) is first-come-first-served and includes a lot of single-
vendor, low-traffic entries (see 2.6). don't spend dictionary budget on
long-tail Provisional entries with near-zero real-world frequency.

2.1 web transport (highest traffic share, by far)
    http, https, ws, wss
    — near-zero entropy in scheme; almost all shortener traffic lives here;
      the authority (domain) is the single highest-value dictionary target.

2.2 mail & messaging
    mailto (RFC 6068), tel (RFC 3966), sms (RFC 5724), im (RFC 3860),
    xmpp (RFC 5122), matrix, sip / sips (RFC 3261), msrp / msrps (RFC 4975)

2.3 file transfer & remote access
    ftp (RFC 1738), sftp, file (RFC 8089), nfs (RFC 2224), ssh, rsync,
    smb, svn, git

2.4 content-addressed / decentralized (path IS a hash — see §5)
    ipfs, ipns (Protocol Labs), dat, hyper, ar (Arweave), ens (Ethereum
    Name Service), ssb (Secure Scuttlebutt), cabal, swh (Software Heritage
    — content-addressed source-code archive), magnet (BitTorrent, informal
    but ubiquitous — xt= parameter carries a base32/hex infohash)

2.5 ledger / crypto payment
    bitcoin, bitcoincash, ethereum, starknet, payto (RFC 8905),
    simpleledger, taler

2.6 app deep-links (unofficial-flavored even when "Provisional")
    android, market, spotify, steam, skype, vscode, web3, intent-style
    schemes generally
    — the ms-* namespace alone is 60+ registered Provisional schemes
      (ms-excel, ms-settings-wifi, ms-remotedesktop, ...), all single-
      vendor, near-zero shared structure. treat as: recognize the prefix
      "ms-" as one dictionary token, then fall through to raw/statistical
      encoding for the specific suffix — enumerating all 60 individually
      is not worth the dictionary budget.
    — this category is the least standardized, highest-branching part of
      the registry. no cross-vendor grammar to exploit; each app invents
      its own path convention.

2.7 naming / identifier schemes (not transport — pure identifiers)
    urn (RFC 8141), tag (RFC 4151), doi, info (RFC 4452), did
    (Decentralized Identifiers, W3C) — these have a fully specified,
    narrow grammar (e.g. urn:NID:NSS) and are excellent grammar-compression
    targets: small alphabet, fixed structure, low real entropy once the
    namespace identifier (NID) is dictionary-matched.

2.8 real-time / signaling / IoT
    sip, sips, stun, stuns, turn, turns (RFC 7065), rtsp / rtsps / rtspu,
    coap / coaps (+tcp/+ws variants, RFC 7252 / 8323) — IoT-oriented,
    growing category, worth tracking as Mtaagrid-adjacent infra work
    touches IoT.

2.9 historical / deprecated (still appear in old archived links)
    gopher, wais, prospero, fax, videotex, z39.50, filesystem, mailserver
    — low dictionary priority, include only for completeness/robustness
      (decoder should not choke on them, but no need to optimize for them).

2.10 space / delay-tolerant networking (novel, worth flagging)
    dtn (RFC 9171), ipn (RFC 9758 — interplanetary networking, has its own
    sub-registries for allocator IDs and node numbers). low volume today,
    but a genuinely distinct addressing model (node/service pairs rather
    than domain/path) if this space becomes relevant later.


3. entropy profile by category (what's worth dictionary-matching vs modeling)

    category                  scheme entropy   authority entropy   path entropy
    ---------------------------------------------------------------------------
    web http/https             ~0 (2 values)    Zipf-distributed    highly variable,
                                                 (dictionary wins)   platform-dependent
    app deep-link               moderate         small closed        small closed
                                (~300-way)       per-vendor set      per-vendor set
    content-addressed           ~0               scheme-fixed        HIGH — literal
                                                                      content hash,
                                                                      not compressible
    naming (urn/tag/doi)        ~0               NID dictionary      LOW — structured
                                                  (small, closed)     grammar
    ledger/crypto               ~0               n/a                 HIGH — address is
                                                                      a hash/pubkey

takeaway: the only category where the *path* itself is fundamentally
incompressible is content-addressed and ledger schemes — because the path
IS the payload's cryptographic identity, by design. no dictionary or model
trained on URL structure will ever help there; that's the honest floor
discussed in the prior research pass, made concrete per-category.


4. query parameter taxonomy

query strings are the least standardized layer (see §1) and the largest
practical compression opportunity, because a huge fraction of real-world
query strings are pure attribution/tracking boilerplate with zero
resource-identifying value — same keys, same structure, different
low-entropy values, repeated across billions of URLs.

4.1 tracking / attribution (closed, slowly-growing vocabulary — prime
    static dictionary target; these are appended by platforms, not chosen
    by the person sharing the link, so they're maximally predictable):

    utm_source, utm_medium, utm_campaign, utm_term, utm_content   (Google
      Analytics / Urchin legacy, the "UTM" family)
    gclid, gclsrc, wbraid, gbraid, dclid                          (Google Ads)
    fbclid                                                        (Meta/Facebook,
                                                                    undocumented,
                                                                    launched ~2018)
    msclkid                                                       (Microsoft Ads)
    twclid                                                        (X/Twitter Ads)
    ttclid                                                        (TikTok Ads)
    li_fat_id                                                     (LinkedIn Ads)
    mc_cid, mc_eid                                                (Mailchimp)
    _ga, _gl                                                      (GA client/linker)
    _hsenc, _hsmi                                                 (HubSpot)
    mkt_tok                                                       (Marketo)
    igshid, igsh                                                  (Instagram share)
    vero_id                                                       (Vero email)
    oly_anon_id                                                   (Olytics)
    epik                                                          (Pinterest)
    si                                                             (generic "share id",
                                                                    YouTube/Spotify)
    dm_i                                                          (Dot Digital email)
    mkevt, mkcid, mkrid, campid, toolid, customid                  (eBay affiliate)
    WT.mc_id, WT.nav                                               (WebTrends legacy)
    hootPostID                                                     (Hootsuite)
    mtm_*, matomo_*, piwik_*                                       (self-hosted
                                                                    analytics — worth
                                                                    a wildcard token
                                                                    given Femar's
                                                                    self-host bias)
    _branch_match_id                                               (Branch.io deep
                                                                    links)

    design note: because these carry no resource-identifying information,
    a shortener CAN legitimately choose to strip them entirely (many
    production shorteners do, framed as "clean sharing") rather than
    encode them — that's a product decision, not a compression one, but
    worth deciding explicitly since it changes the entropy budget a lot.
    if preserved, they compress extremely well: known key names → dictionary
    tokens; values are usually short alphanumeric IDs → residual entropy
    coding.

4.2 pagination / UI state (low entropy, small value domains)
    page, offset, limit, cursor, sort, order, view, tab

4.3 content selectors (HIGH information value — never drop, never treat
    as boilerplate)
    v=            (YouTube video id)
    id=, p=       (generic resource selectors)
    q=            (search query — free text, genuinely high entropy)
    /status/      (X/Twitter path segment, not query, but same role)

4.4 security-relevant (must round-trip byte-exact, never touch)
    token, sig, signature, expires, nonce
    state, code   (OAuth 2.0 flow — RFC 6749)


5. identifier encoding formats found in paths — the actual residual entropy

after scheme + domain + tracking-param stripping, what's usually left in a
path is a platform-issued identifier. its *format* tells you how much of
its width is real entropy vs structural padding — critical for the static
model described in the prior research pass, since a model that recognizes
"this looks like a Snowflake ID" can spend far fewer bits on the timestamp
field than its raw width would suggest.

    format       total bits   structure                          text form
    --------------------------------------------------------------------------
    UUID v4      128          122 random, no timestamp            36 chars,
                                                                    hex+dashes,
                                                                    ~3.5 bit/char
                                                                    (re-encode to
                                                                    base64url/66 →
                                                                    22 chars)
    UUID v7      128          48-bit ms timestamp + ~74 random/
                              counter, time-ordered (RFC 9562)     36 chars hex
    ULID         128          48-bit ms timestamp + 80 random,
                              Crockford base32                     26 chars,
                                                                    lexicographically
                                                                    sortable
    Snowflake    64           41-bit timestamp + 10-bit worker id
                              + 12-bit sequence (original Twitter
                              layout; other platforms vary the
                              split but keep the shape)             up to 19 decimal
                                                                    digits
    KSUID        160          32-bit timestamp + 128 random         27 chars base62
    NanoID       ~126         fully random, URL-safe alphabet,
                              no structure at all — designed
                              explicitly for short URLs/tokens      21 chars default
    bit.ly/yt-style
    opaque token  variable    platform-internal, effectively
                              opaque once observed externally       varies (YouTube
                                                                    video ids: 11
                                                                    chars, base64url-
                                                                    like alphabet)

implication for the model: UUIDv4 and NanoID are, by explicit design,
closest to true Kolmogorov-incompressible randomness — the model should
recognize the pattern fast and not waste cycles trying to find structure
that isn't there. Snowflake / ULID / UUIDv7 are NOT actually random in
their full width — the timestamp field is low-entropy (narrow, near-
monotonic range for any given corpus time window) even without knowing
the issuing platform's internal state. that's free compression available
to a generic model, not just to a platform that owns its own IDs.


6. how this atlas feeds the compressor (crosswalk)

    atlas section          → compressor component
    -----------------------------------------------
    §1 generic syntax       → bijective base alphabet choice (unreserved set)
    §2 scheme taxonomy       → grammar dictionary, tier-1 tokens (scheme+authority)
    §3 entropy profile       → routing logic: which paths get dictionary vs
                              raw/statistical treatment
    §4.1 tracking params     → grammar dictionary, tier-2 tokens (query boilerplate)
    §4.3/4.4 content/security → "never touch" list, always full-fidelity encode
    §5 id formats             → static model features (timestamp-field detection,
                              structure-aware bit allocation)


7. open items / next passes

  - pull real frequency data: which schemes/domains/params actually dominate
    traffic Femar's systems would see (Kenyan market skew — M-Pesa deep
    links, local marketplace domains, WhatsApp share links) vs generic
    global corpora. atlas above is structural/global; needs a frequency
    layer on top before it can train the static model meaningfully.
  - WhatsApp (wa.me / api.whatsapp.com) and other messaging-app share-link
    conventions aren't IANA schemes (they're https paths on owned domains)
    — worth a dedicated §2.11 once we pull real examples, since wa-core is
    active work.
  - build the actual per-platform path-template list referenced in §3
    ("small closed per-vendor set") — this atlas notes the category exists
    but doesn't yet enumerate YouTube/GitHub/Twitter/Instagram/M-Pesa path
    grammars. next research pass.
  - IANA registry text pulled 2026-08-19, last-updated stamp on that date —
    re-fetch before any training run to catch new entries.
