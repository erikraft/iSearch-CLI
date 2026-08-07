import re

def verify_city_presence():
    html_files = [
        "scripts/index.html",
        "scripts/donation.html"
    ]

    js_rust_files = [
        "scripts/js/pix.js",
        "scripts/js/main.js",
        "src/pix.rs"
    ]

    # 1. Verify "PERUIBE" is NOT visually present in HTML files
    # We check if there's any visually displayed text with "PERUIBE".
    # For HTML files, we check if "PERUIBE" is inside tag contents, specifically <div><strong>City:</strong> PERUIBE</div>
    # Actually, let's be extremely safe: no visual display of "PERUIBE" in HTML (which means no "PERUIBE" in HTML at all since there's no scripting in the HTML itself).
    print("--- Verifying HTML files ---")
    for filepath in html_files:
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()
            # Find any visual occurrences of "PERUIBE" (case-insensitive)
            # We want to make sure it doesn't appear as text content.
            # To be absolutely sure, let's make sure "PERUIBE" is not present in the HTML files at all.
            matches = list(re.finditer("PERUIBE", content, re.IGNORECASE))
            if len(matches) > 0:
                print(f"[FAIL] 'PERUIBE' was found in {filepath}!")
                for m in matches:
                    start = max(0, m.start() - 30)
                    end = min(len(content), m.end() + 30)
                    print(f"       Context: ...{content[start:end]}...")
                return False
            else:
                print(f"[PASS] 'PERUIBE' is completely absent from {filepath}.")

    # 2. Verify "PERUIBE" IS present in JavaScript and Rust files (for PIX payload compliance)
    print("\n--- Verifying JS and Rust files ---")
    for filepath in js_rust_files:
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()
            matches = list(re.finditer("PERUIBE", content))
            if len(matches) == 0:
                print(f"[FAIL] 'PERUIBE' is missing in generator file {filepath}!")
                return False
            else:
                print(f"[PASS] 'PERUIBE' is correctly present in {filepath} (found {len(matches)} occurrences).")

    print("\nVerification completed successfully!")
    return True

if __name__ == "__main__":
    import sys
    success = verify_city_presence()
    if not success:
        sys.exit(1)
    sys.exit(0)
