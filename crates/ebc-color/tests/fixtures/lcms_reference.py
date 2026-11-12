# Usage: python3 lcms_reference.py lcms-fogra39.tsv
# Generates crates/ebc-color/tests/fixtures/lcms-fogra39.tsv with Pillow's ImageCms
# (Little CMS) and the bundled basICColor ISO Coated v2 300% profile.
import sys
from PIL import Image, ImageCms
import os
P=os.path.join(os.path.dirname(__file__), '../../../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc')
srgb=ImageCms.createProfile('sRGB'); cmyk=ImageCms.getOpenProfile(P)
lab=ImageCms.createProfile('LAB', 5000)
R=ImageCms.Intent.RELATIVE_COLORIMETRIC; PE=ImageCms.Intent.PERCEPTUAL
BPC=ImageCms.Flags.BLACKPOINTCOMPENSATION
levels=[0,32,64,96,128,160,192,224,255]
cols=[(r,g,b) for r in levels for g in levels for b in levels]
cols+= [(250,240,220),(245,245,250),(230,225,210),(31,58,95),(122,31,43),(30,89,69),(201,162,39),(232,220,194),(20,20,20),(35,31,32),(13,13,13),(200,30,30)]
out=open(sys.argv[1],'w')
out.write('# intent\tr g b\tlcms C M Y K (8-bit)\tlcms Lab (A2B1 of that CMYK, D50)\n')
back=ImageCms.buildTransform(cmyk,lab,'CMYK','LAB',renderingIntent=R)
for name,intent,flags in [('relative_bpc',R,BPC),('perceptual',PE,0)]:
    t=ImageCms.buildTransform(srgb,cmyk,'RGB','CMYK',renderingIntent=intent,flags=flags)
    im=Image.new('RGB',(len(cols),1)); im.putdata(cols)
    c=ImageCms.applyTransform(im,t)
    l=ImageCms.applyTransform(c,back)
    for rgb,k,lv in zip(cols,list(c.get_flattened_data()),list(l.get_flattened_data())):
        L=lv[0]*100/255; a=lv[1]-128 if lv[1]<128 else lv[1]-128; b=lv[2]-128
        # Pillow's LAB mode stores a/b as signed bytes offset by 128
        out.write(f"{name}\t{rgb[0]} {rgb[1]} {rgb[2]}\t{k[0]} {k[1]} {k[2]} {k[3]}\t{L:.2f} {lv[1]-128} {lv[2]-128}\n")
