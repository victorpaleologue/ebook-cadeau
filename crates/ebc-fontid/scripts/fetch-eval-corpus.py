#!/usr/bin/env python3
"""Downloads the font corpus of the ebc-fontid evaluation (measurement only, never shipped).

    python3 -I crates/ebc-fontid/scripts/fetch-eval-corpus.py [OUT]

OUT defaults to target/fontid-corpus (git-ignored). For every family listed in
eval-corpus.txt it fetches, from raw.githubusercontent.com/google/fonts, the licence file
(ofl/<dir>/OFL.txt or apache/<dir>/LICENSE.txt), METADATA.pb (for the category) and the
listed font files into OUT/<dir>/. A family whose licence file cannot be fetched is
skipped entirely, so that every font of the corpus sits next to its licence. Existing
files are kept, so the script can be re-run to complete a partial download.
"""
import os
import subprocess
import sys
import urllib.parse

BASE = "https://raw.githubusercontent.com/google/fonts/main"
LICENCE_FILE = {"ofl": "OFL.txt", "apache": "LICENSE.txt"}


def fetch(url, dest):
    if os.path.exists(dest) and os.path.getsize(dest) > 0:
        return True
    tmp = dest + ".part"
    r = subprocess.run(["curl", "-sS", "-f", "-m", "60", "-o", tmp, url])
    if r.returncode != 0 or not os.path.exists(tmp) or os.path.getsize(tmp) == 0:
        if os.path.exists(tmp):
            os.remove(tmp)
        return False
    os.replace(tmp, dest)
    return True


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, "..", "..", "..", "target", "fontid-corpus")
    fonts = missed = 0
    with open(os.path.join(here, "eval-corpus.txt"), encoding="utf-8") as fh:
        for line in fh:
            if not line.strip() or line.startswith("#"):
                continue
            area, family, files = line.split()
            if area not in LICENCE_FILE:
                continue
            folder = os.path.join(out, family)
            os.makedirs(folder, exist_ok=True)
            url = f"{BASE}/{area}/{family}"
            if not fetch(f"{url}/{LICENCE_FILE[area]}", os.path.join(folder, LICENCE_FILE[area])):
                print(f"skipped {area}/{family}: no licence file", flush=True)
                continue
            fetch(f"{url}/METADATA.pb", os.path.join(folder, "METADATA.pb"))
            for name in files.split("|"):
                if fetch(f"{url}/{urllib.parse.quote(name)}", os.path.join(folder, name)):
                    fonts += 1
                else:
                    missed += 1
                    print(f"missed {area}/{family}/{name}", flush=True)
    print(f"{fonts} font files in {os.path.abspath(out)}, {missed} missed", flush=True)


if __name__ == "__main__":
    main()
