#!/usr/bin/env python3
"""cutout.py SRC DST LO HI: cut a logo out of a dark, blurred background.

Background = pixels connected to the image border whose brightest channel
is in LO..HI (the photo behind the logo); the logo's black outline (below
LO) and light fill (above HI) stop the flood.  Writes DST as a square RGBA
PNG, cropped to the logo with an 8-pixel margin.  Needs Pillow and numpy.
Used by desktop/make-icons.sh."""
import sys
from collections import deque
from PIL import Image, ImageFilter
import numpy as np
src, dst, lo, hi = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
im = Image.open(src).convert("RGB"); a = np.asarray(im).astype(int); mx = a.max(2)
cand = (mx >= lo) & (mx <= hi)
H, W = cand.shape
bg = np.zeros_like(cand)
q = deque()
for x in range(W):
    for y in (0, H - 1):
        if cand[y, x] and not bg[y, x]: bg[y, x] = True; q.append((y, x))
for y in range(H):
    for x in (0, W - 1):
        if cand[y, x] and not bg[y, x]: bg[y, x] = True; q.append((y, x))
while q:
    y, x = q.popleft()
    for dy, dx in ((1,0),(-1,0),(0,1),(0,-1)):
        ny, nx = y + dy, x + dx
        if 0 <= ny < H and 0 <= nx < W and cand[ny, nx] and not bg[ny, nx]:
            bg[ny, nx] = True; q.append((ny, nx))
alpha = Image.fromarray(((~bg) * 255).astype(np.uint8), "L")
# remove background specks (dark JPEG noise) and smooth the edge
alpha = alpha.filter(ImageFilter.MinFilter(5)).filter(ImageFilter.MaxFilter(5))
alpha = alpha.filter(ImageFilter.GaussianBlur(0.7))
out = im.convert("RGBA"); out.putalpha(alpha)
out = out.crop(out.getbbox())
w, h = out.size; n = max(w, h) + 16
sq = Image.new("RGBA", (n, n), (0, 0, 0, 0)); sq.paste(out, ((n - w) // 2, (n - h) // 2))
sq.save(dst, "PNG"); print(dst, sq.size)
