import os
import sys
import csv

def main():
    base_dir = os.path.dirname(os.path.abspath(__file__))
    list_path = os.path.join(base_dir, "list.csv")
    common_path = os.path.join(base_dir, "common.csv")
    
    csv_out = os.path.join(base_dir, "urls_5000.csv")
    tsv_out = os.path.join(base_dir, "urls_5000.tsv")
    txt_out = os.path.join(base_dir, "urls_5000.txt")

    target_count = 5000
    rows = []

    # If list.csv is available, extract the exact first 5,000 rows generated from common.csv
    if os.path.exists(list_path):
        print(f"Extracting first 5,000 URLs from {list_path}...")
        with open(list_path, "r", encoding="utf-8", errors="ignore") as f:
            reader = csv.reader(f)
            header = next(reader, None)
            for row in reader:
                if not row or len(row) < 3:
                    continue
                rows.append(row)
                if len(rows) >= target_count:
                    break
    else:
        # Fallback to generate dynamically from common.csv
        print(f"Generating first 5,000 URLs from {common_path}...")
        import random, string
        schemes = ["https://", "https://www.", "http://", "http://www.", "https://api.", "https://app.", "https://blog.", "https://m.", "https://dev.", "https://shop.", "https://portal.", "https://auth.", "https://cdn.", "https://docs.", "https://secure.", "https://media."]
        path_templates = ["watch?v=", "dp/", "r/posts/", "status/", "pull/", "blob/main/src/", "search?q=", "user/profile/", "wiki/articles/", "app/view/", "item/catalog?id=", "questions/tagged/", "track/event/", "article/2026/08/", "news/top/content/", "shop/products/deals/", "feed/trending/", "explore/tags/", "api/v2/endpoints/", "comments/thread/"]
        words = ["rust", "systems", "compression", "algorithms", "performance", "store", "database", "memory", "succinct", "delta", "entropy", "hashing", "graph", "network", "crypto", "distributed", "concurrency", "optimization", "benchmark", "analysis", "compiler", "parser", "lexer", "ast", "token", "vector", "matrix", "stream", "payload", "pipeline", "profile", "manifest", "cluster", "shard"]
        sources = ["google", "twitter", "linkedin", "reddit", "youtube", "email", "direct", "facebook", "tiktok", "bing", "github"]
        mediums = ["cpc", "social", "organic", "newsletter", "referral", "affiliate", "banner", "feed"]
        campaigns = ["summer_sale", "newsletter_aug2026", "hero_cta", "winter_event", "launch_v2", "weekly_digest", "social_ad", "retargeting_global", "partner_promo"]
        chars_alnum = string.ascii_letters + string.digits
        chars_hex = "0123456789abcdef"
        random.seed(42)
        rand_alnum_16 = [''.join(random.choices(chars_alnum, k=16)) for _ in range(1000)]
        rand_alnum_32 = [''.join(random.choices(chars_alnum, k=32)) for _ in range(1000)]
        rand_hex_12 = [''.join(random.choices(chars_hex, k=12)) for _ in range(1000)]
        rand_hex_24 = [''.join(random.choices(chars_hex, k=24)) for _ in range(1000)]
        rand_slugs = ['-'.join(random.choices(words, k=3)) for _ in range(1000)]

        with open(common_path, "r", encoding="utf-8", errors="ignore") as f:
            count = 0
            for line in f:
                parts = line.strip().split(",", 1)
                if len(parts) != 2 or not parts[1].strip():
                    continue
                count += 1
                domain = parts[1].strip().lower()
                idx = count % 1000
                scheme = random.choice(schemes)
                path = random.choice(path_templates)
                slug = rand_slugs[idx]
                token = rand_alnum_16[idx]
                hex_id = rand_hex_12[idx]
                if "watch?v=" in path:
                    path_part = f"{path}{rand_alnum_16[idx][:11]}"
                elif "dp/" in path:
                    path_part = f"{path}B0{rand_alnum_16[idx][:8]}?ref=nb_sb_{hex_id}"
                elif "search?q=" in path:
                    path_part = f"{path}{slug}+{token}"
                elif "item" in path:
                    path_part = f"{path}{count}&slug={slug}-{hex_id}"
                else:
                    path_part = f"{path}{slug}-{token}_{hex_id}"
                url = f"{scheme}{domain}/{path_part}"
                if random.random() < 0.65:
                    src = random.choice(sources)
                    med = random.choice(mediums)
                    camp = random.choice(campaigns)
                    gclid = rand_alnum_32[idx]
                    session = rand_hex_24[idx]
                    sep = "&" if "?" in url else "?"
                    url = f"{url}{sep}utm_source={src}&utm_medium={med}&utm_campaign={camp}&gclid={gclid}&session_id={session}&timestamp=1755600000"
                rows.append([str(count), domain, url])
                if len(rows) >= target_count:
                    break

    # Write CSV
    with open(csv_out, "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["id", "domain", "url"])
        writer.writerows(rows)

    # Write TSV
    with open(tsv_out, "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f, delimiter="\t")
        writer.writerow(["id", "domain", "url"])
        writer.writerows(rows)

    # Write TXT
    with open(txt_out, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(f"{r[2]}\n")

    print(f"Successfully generated {len(rows):,} rows in {csv_out}, {tsv_out}, and {txt_out}")

if __name__ == "__main__":
    main()
