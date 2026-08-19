tools.md
algorithms and encoding stack — lightweight, from-scratch where reasonable
status: living document
companion: idea.md (why), store.md (where bytes live), profile.md (what
drives the statistical model), url-protocol-atlas.md (the corpus this is
trained against)


0. philosophy

no heavy packages. if a dependency would pull in more than the specific
algorithm needed, rewrite that algorithm from scratch instead. total code
budget: under 1MB. this is generous, not tight — every core piece below has
working precedent at a fraction of that size (lpaq-family compressors have
shipped as a few hundred KB total, including the neural mixer). metadata
(dictionaries, trained weights, frequency tables) is explicitly exempt from
the 1MB budget and lives separately as versioned profile artifacts
(profile.md, objects.md) — code stays small, metadata is allowed to grow.

this stack is purpose-built for one workload: short strings (URLs, tens to
low hundreds of bytes) with heavy structural redundancy at the protocol/
domain/tracking-param level. it is not a general-purpose compressor. do not
add generality "in case it's useful later" — every added axis of generality
is a place where the 1MB budget and the speed target both erode.


1. the encoding stack, in order

    raw url bytes
      -> normalize (IRI/percent-decoding — see url-protocol-atlas.md §1)
      -> dictionary match against profile's structural table
           (scheme+authority templates, tracking-param names, id-format
           recognizers)
      -> grammar/SLP transform on the residual (collapse repeated
           substructure)
      -> entropy code the remaining residual, driven by the profile's
           static statistical model
      -> bijective base encoding of the final bitstream
      -> short keyed integrity/version tag appended

each stage is described below with what it is, why this one and not an
alternative, and the size/speed cost.


2. bijective numeration — the base layer

standard positional base-N has a leading-zero ambiguity problem. bijective
base-N (digits 1..N rather than 0..N-1, the scheme spreadsheet columns use)
gives every non-negative integer exactly one representation, no padding, no
edge cases, O(1) length. alphabet: the RFC 3986 unreserved set (66
characters) so the output round-trips through any URL context without
further escaping. this is a few dozen lines of code, not a library.


3. grammar / straight-line-program transform — structural redundancy

collapses repeated substructure (repeated path segments, `&key=` boundaries,
percent-encoding artifacts) before entropy coding sees the string. finding
the *smallest* possible grammar for a string is proven NP-hard, so this uses
a fast heuristic, not an optimal solver: Re-Pair-style — recursively replace
the most frequent adjacent symbol pair with a new symbol, linear time,
reaches high-order compression especially on repetitive input. complements
entropy coding rather than competing with it: grammar transform removes
*structural* redundancy, entropy coding removes *statistical* redundancy
from what's left.

note from url-protocol-atlas.md §3: this stage does nothing for content-
addressed paths (ipfs/magnet/crypto-address schemes) where the path is
itself a hash — there is no structure there to find, by design. the router
should recognize these schemes and skip straight to raw entropy coding
rather than wasting cycles on a grammar pass that cannot help.


4. entropy coding — asymmetric numeral systems (ANS), not Huffman or
   naive arithmetic coding

ANS is the right choice specifically because it hits arithmetic-coding
compression ratios at close to Huffman-coding speed — decode is table
lookups and additions on a single natural-number state, not the range
arithmetic that makes classic arithmetic coding slower. this is the
correct primitive given the stated "fastest, absolute speed" priority; it
is also what the trained statistical model (below) feeds into.

rANS variant (range/tabled ANS) specifically, for the fast table-driven
decode path. implementation: a few hundred lines, no external crate
required, though a focused rANS crate is a reasonable narrow dependency if
it saves meaningful implementation risk — the "no heavy packages" rule
targets frameworks that do more than needed, not a tightly-scoped single-
purpose primitive.


5. the statistical model driving the entropy coder — static, not live

this is the "ML-like" piece, sized correctly for the constraint (see
profile.md for how profiles package this).

what it is NOT: a neural network doing inference on the hot path, or
online learning happening live during a single ~60-byte encode. there
isn't enough content in one URL for online adaptation to converge before
the string ends — that's a real limitation of context-mixing techniques,
not a reason to avoid the technique, just a reason to apply it correctly.

what it IS: the same class of technique used by lpaq-family compressors —
a small logistic mixing model (order-N context predictors combined via a
weighted average in the logistic domain) — but *trained once offline*
against a large corpus (url-protocol-atlas.md-informed), producing frozen
weights that ship as part of a profile. optimizing this kind of predictive
model's loss is mathematically equivalent to optimizing the code length
under entropy coding, so a sharper offline-trained model directly means a
shorter code, with zero runtime training cost. weights are refreshed by
publishing a new profile version (objects.md), never mutated live.

fallback for the generic (untrained-for-this-domain) case: an order-2/3
context frequency table is a weaker but much simpler and smaller model,
appropriate as the always-available default profile before any fine-tuning
has happened.


6. hashing — BLAKE3, uniformly, everywhere in the system

used for: profile version identity (profile.md), object content-addressing
(objects.md), integrity tags appended to encoded codes. one algorithm,
one implementation, everywhere — no reason to also carry SHA-256 anywhere
in this system. BLAKE3 is 4-10x faster than SHA-256 on typical hardware
and, unlike SHA-256's inherently sequential construction, is a genuine
Merkle tree internally, so it parallelizes natively across cores — which
matters given store.md's shard-per-core design and objects.md's Merkle-DAG
structure both want a hash function with that shape anyway. no part of
this system needs SHA-256-specific properties (e.g. NIST/FIPS compliance);
if that requirement ever appears from an external integration, it would be
scoped narrowly to that integration point, not adopted system-wide.


7. what got explicitly rejected, and why (don't re-propose without new
   information)

  - general-purpose compression libraries (zstd, brotli, etc. as
    dependencies): they solve a broader problem than this one, at a size
    and complexity cost this project doesn't need. the *ideas* from zstd
    (dictionary training, ZDICT's "no universal dictionary" finding) are
    used as design precedent in profile.md; the library itself is not a
    dependency.
  - full arithmetic coding instead of ANS: correct compression ratio, but
    slower in practice due to range-arithmetic operations vs ANS's table
    lookups — wrong tradeoff given the stated speed priority.
  - live/online neural inference per encode: no time to converge on a
    ~60-byte string (section 5); would also blow the size/speed budget for
    no benefit at this string length.
  - a real neural network framework (even a small one) as a dependency:
    the logistic-mixing model in section 5 is a few hundred lines of
    hand-rolled weighted-average logic, not a network requiring a
    framework — pulling in an actual ML framework for this would be
    exactly the kind of heavy package this project is deliberately
    avoiding.
