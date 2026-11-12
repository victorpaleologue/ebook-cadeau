# ICC profiles

From <https://github.com/saucecontrol/Compact-ICC-Profiles>, dedicated to the public domain
under **CC0 1.0** (`LICENSE-CC0.txt`).

| File | Colour space | Use |
|---|---|---|
| `sRGB-v2-micro.icc`, `sRGB-v4.icc` | sRGB | tagging RGB cover images |
| `CGATS001Compat-v2-micro.icc` | CMYK (CGATS TR 001 / SWOP compatible) | small redistributable CMYK destination profile for tests and as a fallback output intent |

European printers normally expect FOGRA39 (ISO Coated v2) or FOGRA51 (PSO Coated v3). Those
ECI profiles are free to use but have their own licence; see
`docs/research/03-print-requirements.md` before bundling them. The PDF writer accepts any
profile supplied by the caller.
