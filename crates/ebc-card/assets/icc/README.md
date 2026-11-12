# Output-intent profile bundled with `ebc-card`

| File | What | Licence |
|---|---|---|
| `ISOcoated_v2_300_bas.icc` | "ISO Coated v2 300% (basICColor)": CMYK **output-device** profile (ICC class `prtr`, version 2.1) for the FOGRA39L characterisation, total ink limit 300 %. 1 052 612 bytes, SHA-256 `b424c77f40c3423c925536f8ae08634985ccd0fe80eb253d5d229197ded7e886` | zlib/libpng licence, Copyright (c) 2007-2010 basICColor GmbH — `ISOcoated_v2_300_bas.LICENSE.txt` |

## Provenance

Downloaded unmodified on 2026-11-12 from the GitHub mirror of the Scribus sources:

* repository: <https://github.com/scribusproject/scribus> (branch `master`)
* profile: `resources/profiles/ISOcoated_v2_300_bas.icc`
* licence file: `resources/profiles/ISOcoated_v2_300_bas.txt` (copied here verbatim as
  `ISOcoated_v2_300_bas.LICENSE.txt`)

The zlib licence allows use, alteration and redistribution, commercial use included,
provided the origin is not misrepresented and the notice travels with the file. Keep the
licence file next to the profile wherever the profile is copied or served.

## Caveats (not verified by a lawyer)

* The profile's own `cprt` tag still carries the string written by the tool that built it
  ("basICColor CMYKick v1.2 - Copyright (c) 2006-2007 Color Solutions, All Rights
  Reserved."). The redistribution grant is the licence file that basICColor's profile is
  distributed with in Scribus; Debian's `icc-profiles-free` records the same zlib terms
  for basICColor GmbH / Color Solutions profiles. Nothing here was confirmed with
  basICColor directly.
* This is basICColor's build of the FOGRA39L characterisation, not ECI's
  `ISOcoated_v2_300_eci.icc`. Both describe the same printing condition (identifier
  `FOGRA39`); a printer that insists on the ECI file must be given it through
  `CardRequest::output_profile`. The ECI profiles are **not** bundled: their licence text
  could not be read from this build environment
  (`docs/research/03-print-requirements.md` §3.5).
* Printers that run FOGRA51 (PSO Coated v3) or FOGRA52 (uncoated) need their own profile,
  again through `CardRequest::output_profile`.

The repository-level `assets/icc/CGATS001Compat-v2-micro.icc` (CC0) remains the last-resort
fallback when the `bundled-fogra39` feature is off: it is an input-class (`scnr`) profile
for a US condition, and `ebc-card` then reports `CardWarning::FallbackOutputProfile`.
