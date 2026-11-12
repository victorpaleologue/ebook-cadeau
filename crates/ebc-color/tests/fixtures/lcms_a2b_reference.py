# Usage: python3 lcms_a2b_reference.py lcms-a2b-fogra39.tsv
# Generates crates/ebc-color/tests/fixtures/lcms-a2b-fogra39.tsv with Pillow's ImageCms
# (Little CMS): the Lab (D50) of CMYK colours through the A2B1 table (relative
# colorimetric) of the bundled basICColor ISO Coated v2 300% profile. The CMYK inputs are
# a 5^4 lattice plus the dark builds the ink limiter produces (rich blacks under 240 % and
# 300 %, GCR builds of navy, red, green, brown and purple).
import os
import sys

from PIL import Image, ImageCms

P = os.path.join(os.path.dirname(__file__), '../../../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc')
cmyk = ImageCms.getOpenProfile(P)
lab = ImageCms.createProfile('LAB', 5000)
R = ImageCms.Intent.RELATIVE_COLORIMETRIC
levels = [0, 64, 128, 191, 255]
cols = [(c, m, y, k) for c in levels for m in levels for y in levels for k in levels]


def pct(*v):
    return tuple(round(x * 255 / 100) for x in v)


cols += [
    pct(60, 40, 40, 100), pct(75, 68, 67, 90), pct(50, 50, 50, 100), pct(30, 30, 30, 100),
    pct(100, 85, 30, 60), pct(90, 75, 25, 50), pct(80, 60, 20, 80), pct(40, 30, 0, 100),
    pct(20, 100, 80, 40), pct(10, 90, 70, 60), pct(85, 30, 80, 50), pct(60, 10, 50, 85),
    pct(40, 65, 80, 55), pct(20, 50, 65, 80), pct(70, 90, 20, 40), pct(50, 75, 0, 70),
    pct(0, 0, 0, 100), pct(0, 0, 0, 50), pct(100, 100, 100, 100), pct(5, 5, 5, 5),
]
out = open(sys.argv[1], 'w')
out.write('# C M Y K (8-bit)\tlcms Lab through A2B1 (8-bit Lab, D50)\n')
t = ImageCms.buildTransform(cmyk, lab, 'CMYK', 'LAB', renderingIntent=R)
im = Image.new('CMYK', (len(cols), 1))
im.putdata(cols)
l = ImageCms.applyTransform(im, t)
data = l.get_flattened_data() if hasattr(l, 'get_flattened_data') else l.getdata()
for k, lv in zip(cols, list(data)):
    L = lv[0] * 100 / 255
    out.write(f"{k[0]} {k[1]} {k[2]} {k[3]}\t{L:.2f} {lv[1] - 128} {lv[2] - 128}\n")
