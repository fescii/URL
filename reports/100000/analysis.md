# Scale Analysis Report: 100,000 URLs

This document provides in-depth technical analysis for the ingestion and storage benchmark of **100,000 URLs**.

## 1. Key Performance Metrics

| Metric | Value | Description |
|:---|:---|:---|
| **Total Processed URLs** | `100,000` | Ingested from real-world `list.csv` |
| **Ingested Raw Volume** | `18663.95 KB` (19111886 B) | Uncompressed UTF-8 input size |
| **Stored Disk Footprint** | `16747.10 KB` (17149033 B) | Append-only sharded Bitcask logs |
| **Disk Savings Ratio** | `10.27%` | Net storage reduction vs raw URLs |
| **In-Memory Index (Mutable)** | `4074517 B` (~40.7 B/key) | Active hash index during ingestion |
| **In-Memory Index (Sealed)** | `595508 B` (**5.96 B/key**) | Minimal Perfect Hash (MPHF) bitvectors |
| **Ingestion Throughput** | `2,292 URLs/sec` | Batch processing pipeline rate |
| **Average Lookup Latency** | `3.29 µs` | Direct zero-copy mmap point lookup |

## 2. Multi-Tier Deduplication & Storage Architecture

### A. Myers Bit-Parallel Positional Delta (`Record` & `Delta`)
- Payloads sharing lexical prefixes and structural anchors are deduplicated using 128-bit match bitmasks.
- Characters matching centroid anchors consume **0 bytes** in the substitution diff stream.
- Non-matching characters are packed into compact byte streams alongside delta headers.

### B. Fast Static Symbol Table (FSST) Anchor Dictionaries
- Anchor strings are compressed using shard-level 256-symbol static tables, achieving $> 2.5\text{ GB/s}$ decompression throughput.
- Anchor clustering uses 32-bit MinHash sketches to group structurally similar URLs into shared centroids.

### C. Succinct Minimal Perfect Hash Table (`Succinct`)
- When sealed, shard indices transition from open-addressing hash maps to a two-level MPHF structure.
- 16-bit pilot seeds and 8-bit XOR fingerprints provide collision-free $O(1)$ lookups with **$< 9\text{ Bytes/key}$** RAM usage.

## 3. Sample Encoding Comparisons

| ID | Original URL (Sample) | Tier 1/1.5 Code | Tier 2 Shortcut | Raw | T1/1.5 | T2 | Savings |
|:---|:---|:---|:---|:---:|:---:|:---:|:---:|
| 1 | `https://shop.google.com/user/profile/distr...` | `0CEbn5mgfiOdL~1zn6DNmt-Znr88EDGMFLXluKIXALluOM6DfX9.7f4pr55u9fQ9~30f4ylzjlIrruMSLdGW1JLxYl7Eh8N0aHqIfO7fi.Z0BbFDS5wBZVZa5D~F9ysmrU29jFwOzbzOQbhb~5FMGcsJBvKhOiBXO2SbtEw.s_KVKuOX8QbL0y-3bb5IqV.` | `Urh7.` | 242 B | 191 B | 5 B | **-97.9%** |
| 2 | `https://shop.cloudflare.com/wiki/articles/...` | `0fCq.UH~qH_GkRy.tBPgOwv-24-LWgiZuI7r3Ndp~D11YWmKEWaXhBfmJTfspV6~1QepVkh6uG_KdVjntZ7WEzFoBbWz.bHqdBqxxUd_QJWO5zU19GwQB8Il3JXuWvtYMElVc11C7cayqGaGY8RLyRr3Ym-.AA9w5stuj~f8KxScnEMDQUsp3s5P73l5VTDYGtpSBXe2L_CBH` | `Q4.10` | 265 B | 205 B | 5 B | **-98.1%** |
| 3 | `https://shop.facebook.com/shop/products/de...` | `0jPymRHPV-IAP0YRW8M2FMbGAnWvyEBcqU4CCjfmrkFMeh2w.KtbskFQ~RfOT~_UY7ec~x5et7C9Ro_Wh0bGqZcMRfUD5E_obCEoZ4PxFmbmSJr5MUXBVXkr7Na-NO6Ka8NNW.VltZ8yb9~UppNqjY5dgvgwGBjLSNj7DRQ-wJhNXM6N~-KSu-JFw-NdSBwuvaFl8KSF.` | `PDVz-` | 252 B | 201 B | 5 B | **-98.0%** |
| 4 | `https://portal.gstatic.com/explore/tags/da...` | `0BHoOanIKb578DTMIl1k3v60N1bwGPIgnMAU8GROReD6O8.4nVG2AxA_t1s.2A84zG-VwJw63QoeeXuM5cjYdvYAL~F2dTjy1KtjIC_ImLiDCXNvEaHPr_-EYV4C2LwvKqF-ZQ4Ik5VXSqtOyLt1W5qkpGJKcyKpLXh8fDxwocpIRlBBm2QgWP7c3DD6V-xvDSSGlIneSDT` | `U6bgE` | 252 B | 203 B | 5 B | **-98.0%** |
| 5 | `https://dev.gtld-servers.net/pull/profile-...` | `0JkurY8sDgYy8gxVHhGgy1JWZgJ82nEtRbz2M8BW3xbwhUP1kIRTBasL2JjW4cDn9P5A59LBe_QMVFzx.0u76tlcioIA_.BgJ6ISo1G6b-6wm6u0X-lYzaDjEmhR2uJI6uN078QSCXVZZv2_S-p6.ebQxz6E1FBjwMSEhMSzTiw5.ivRfmxzdl9kGdwHrHS6qod3` | `NmIqy` | 247 B | 196 B | 5 B | **-98.0%** |
| 6 | `https://portal.googleapis.com/questions/ta...` | `0ASi90aLYp6E3XZAN5J2FF8VaxGcAk~WL4_teeT_NdFQpxRG4laBadYCDLKVAcAfDUmnA9geTzsDBqnj~Mdb` | `RalN-` | 102 B | 84 B | 5 B | **-95.1%** |
| 7 | `https://www.microsoft.com/wiki/articles/le...` | `0VCwl2iIc6.vBcpDi.OZVUdzUZpqHMI2dp3bCiTSdbL7LHfjSoyQszSqQ7S4BOqX21VLr3~XiyKimT56~U` | `Ut.81` | 96 B | 82 B | 5 B | **-94.8%** |
| 8 | `https://m.amazonaws.com/search?q=benchmark...` | `0OSaX9k223HCLpAzXrKly4P6HCeaWBcY40yOqIolWXmbHEcDk_.XciQWuc892dW2vgzuyQ99EMS95SKuNJGA7aAUNz~8mPvb-jQQnbAgG~So_wHQ7fH3-tdlweX6GtlgVyo.Sqak~SLCVMSp4EiUJ3T_Si.mS2_0D3NUy7JdqO92n9wk.bbAssfT` | `V7C78` | 236 B | 184 B | 5 B | **-97.9%** |
| 9 | `https://blog.youtube.com/blob/main/src/con...` | `0Adw4DBXqObtXrrV__p9EqI6LJ.iR1vH22xiYZ3yKdYEQWjIK_0e4MSVh20OYrorJsYj~-209AviZkEZ` | `NzDT9` | 99 B | 80 B | 5 B | **-94.9%** |
| 10 | `http://apple.com/track/event/manifest-data...` | `0e7EBivmy9I993QPo.TM92ACnw43t_omNlaWW9NGV33od_kcrTTfjPxzK2Py9p-u25Tnnt` | `XWazP` | 82 B | 70 B | 5 B | **-93.9%** |
| 11 | `https://media.instagram.com/explore/tags/c...` | `0BaCYqgqwJA-zfS~PS~CXBvtce_jYN..PNVz3q~ifVzg9Tuun8yatRnzu06-7HXy3aXY9SN2YASqfKEn-C~iBCAEjs9ldvwzIfzM.3G267zxuk5.6F.4W9fIFHgCwJ_kgrtl9vedV1dEZz-rwvwd2fGcleY7.RWjNNuzTjyJZ9RFxnI-fg7AVgPcPec.UPonaNa._CF` | `NLwqq` | 256 B | 199 B | 5 B | **-98.0%** |
| 12 | `https://mail.ru/item/catalog?id=12&slug=ma...` | `0BZTjTNjDBdz4KNuXqPOLUbbvSoIV.a7U_2._ZOkDzY57VnT3jkg7c7aIOdA7q0e` | `WzZuS` | 77 B | 64 B | 5 B | **-93.5%** |
| 13 | `https://blog.akamai.net/feed/trending/stor...` | `0ASjSLuvI~Ie.pX3r5SbzKvZxmL52mcb3sfbpXb0PtZIJi5tp4Ho_MZkXMDtserqwIXkEzHMl0BxFQKw-Cgs` | `Zbd~d` | 95 B | 84 B | 5 B | **-94.7%** |
| 14 | `https://api.ezviz7.com/shop/products/deals...` | `0fCvM4U_3Bnul2c2DEMzOidgKOjtMae6UxHly_Nlltqxuumz.Ace.fEBbBGgCF9wWgbM.SOy-~8r_IFnUC8aj37j-9rSt3Oa20.XCr7pDHab_yOlpHh5JI8MS0fpjXmXUJSMgZQotvEi0NW.9~OmvyXsSbHO1PdFMto6Wi7JFwwujaP.gXloNQAASat51tA4mq6NSzFXrqEPz` | `ROT.O` | 256 B | 205 B | 5 B | **-98.0%** |
| 15 | `http://fbcdn.net/user/profile/store-perfor...` | `0FbfRZH5PP6DLlipsVziIIoaau724~Nu-38PSi2WTsHIuvPaXf15gurx8s7QW02AjLojFCH~NA3I2` | `N1ktg` | 86 B | 77 B | 5 B | **-94.2%** |

## 4. Verification & Integrity

- **Lossless Integrity**: 100.00% Verified (All 100,000 sample lookups matched byte-for-byte).
- **Point Query Performance**: Zero-copy mmap reads verified with microsecond latency.
