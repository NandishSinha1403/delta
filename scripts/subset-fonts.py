#!/usr/bin/env python3
"""Builds every code-font file Delta ships (DESIGN.md, "Code font"). Run from the repo root:

    python3 scripts/subset-fonts.py        # needs fontTools + brotli (pip install fonttools brotli)

The one code font is IBM Plex Mono. It has no Greek and few maths symbols, so two Noto fallbacks
fill exactly the characters Plex lacks: Noto Sans Math (Greek, ∪ ∩ ∈ ⊆ ≡ ⊔ ⊢ ▸ …) and Noto Sans
Mono (superscript and modifier letters: ⁺ ⁿ ᵏ ᵐ …). All three are SIL OFL 1.1.

State names in diagrams are set in STIX Two Text italic (OFL), as maths is: a subset with Latin
letters, digits, subscript digits and ASCII punctuation, plus a table of its advance widths for
delta-render (crates/delta-render/src/stix_widths.rs, generated) so states fit their names.

Sources: IBM Plex Mono from web/node_modules/@ibm/plex-mono (version pinned in web/package.json);
the two Noto fonts and STIX Two Text are downloaded from pinned commits and checked against their
SHA-256.
Outputs (all committed):
  web/src/fonts/                 the Pad: the two Noto fallbacks (Plex comes from node_modules)
  site/public/fonts/             Plex Mono 400/500 + both fallbacks, as woff2 (the manual copies these)
  crates/delta-export/fonts/     Plex Mono 400 + both fallbacks, as TTF (exports, deltac, cards)
  web/src/fonts/, crates/delta-export/fonts/   STIX Two Text italic (state names)
"""
import hashlib
import shutil
import sys
import tempfile
import urllib.request
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

REPO = Path(__file__).resolve().parent.parent
PLEX = REPO / "web/node_modules/@ibm/plex-mono/fonts/complete/woff2"
NOTO = "https://raw.githubusercontent.com/notofonts/notofonts.github.io/{sha}/fonts/{name}/hinted/ttf/{name}-Regular.ttf"
NOTO_MATH = ("NotoSansMath", "55773c3eb233b5a0eaa07de6226da189b136b4f0",
             "d51afd5739c7ba6c44fcab35a88160e25dfb69a2d4ad0bd99533f8d894af1f96")
NOTO_MONO = ("NotoSansMono", "8b36ddb234b06c091320e28985a3c96b48ce2596",
             "65b5e2b2c4a1fba9ae8be1f026cb35b03dcb8886d9b2a4147054fde12f7e767d")

STIX = ("https://raw.githubusercontent.com/google/fonts/718c7ee0e83f5e301cc96bbc081c9bb5ade9ed92"
        "/ofl/stixtwotext/STIXTwoText-Italic%5Bwght%5D.ttf",
        "88c0e2e316eaff56eddc9e51e4850317e2a1e490bbf758b2dec4793aedba9c74")
# STIX Two Text italic: ASCII, the prime, subscript digits.
STIX_RANGES = [(0x20, 0x7E), (0x2032, 0x2033), (0x2080, 0x2089)]

# Plex Mono: Latin, punctuation, arrows, maths, box drawing, shapes (no Cyrillic or Vietnamese).
PLEX_RANGES = [(0x20, 0x7E), (0xA0, 0x24F), (0x2000, 0x2BFF), (0xFFFD, 0xFFFD)]
# Noto Sans Math: Greek, letterlike, arrows, maths operators, technical, shapes, maths brackets.
MATH_RANGES = [(0x391, 0x3C9), (0x3D1, 0x3D1), (0x3D5, 0x3D5), (0x3F5, 0x3F5), (0x2100, 0x214F),
               (0x2190, 0x22FF), (0x2300, 0x231F), (0x25A0, 0x25FF), (0x27E6, 0x27FF)]
# Noto Sans Mono: superscripts and subscripts, modifier letters.
SUP_RANGES = [(0x1D2C, 0x1D6A), (0x2070, 0x209F)]


def expand(ranges):
    return {c for lo, hi in ranges for c in range(lo, hi + 1)}


def fetch(name, sha, digest, tmp, url=None):
    path = Path(tmp) / f"{name}.ttf"
    with urllib.request.urlopen(url or NOTO.format(sha=sha, name=name)) as r:
        path.write_bytes(r.read())
    got = hashlib.sha256(path.read_bytes()).hexdigest()
    if got != digest:
        sys.exit(f"{name}: SHA-256 {got} does not match the pinned {digest}")
    return path


def cut(src, out, codepoints, flavor):
    have = set(TTFont(src).getBestCmap())
    options = subset.Options()
    options.flavor = flavor            # "woff2" for the web, None for a TTF
    options.layout_features = ["*"]
    options.name_IDs = ["*"]           # keep the family name: exports select fonts by it
    options.notdef_outline = True
    options.hinting = False            # renderers here don't use TrueType hints; much smaller
    font = subset.load_font(str(src), options)
    sub = subset.Subsetter(options)
    sub.populate(unicodes=sorted(codepoints & have))
    sub.subset(font)
    out.parent.mkdir(parents=True, exist_ok=True)
    subset.save_font(font, str(out), options)
    print(f"{out.relative_to(REPO)}: {len(codepoints & have)} characters, {out.stat().st_size // 1024} KB")


def stix_static(tmp):
    """STIX Two Text italic at weight 400 (the published file is a variable font)."""
    from fontTools.varLib import instancer
    var = fetch("STIXTwoText-Italic-var", None, STIX[1], tmp, url=STIX[0])
    font = instancer.instantiateVariableFont(TTFont(var), {"wght": 400})
    out = Path(tmp) / "STIXTwoText-Italic.ttf"
    font.save(out)
    return out


def stix_widths(src, out):
    """The advance width of every character in the subset, in thousandths of an em."""
    font = TTFont(src)
    cmap = font.getBestCmap()
    upem = font["head"].unitsPerEm
    rows = []
    for c in sorted(expand(STIX_RANGES) & set(cmap)):
        adv = font["hmtx"][cmap[c]][0] * 1000 // upem
        rows.append(f"    ({c:#x}, {adv}),")
    out.write_text(
        "//! Advance widths of STIX Two Text italic (state names), in thousandths of an em.\n"
        "//! Generated by `scripts/subset-fonts.py`; do not edit.\n\n"
        "/// (code point, advance) sorted by code point.\n"
        "pub(crate) const STIX_ITALIC: &[(u32, u16)] = &[\n" + "\n".join(rows) + "\n];\n"
    )
    print(f"{out.relative_to(REPO)}: {len(rows)} widths")


def main():
    plex_regular = PLEX / "IBMPlexMono-Regular.woff2"
    plex_have = set(TTFont(plex_regular).getBestCmap()) & expand(PLEX_RANGES)
    with tempfile.TemporaryDirectory() as tmp:
        math = fetch(*NOTO_MATH, tmp)
        mono = fetch(*NOTO_MONO, tmp)
        # The fallbacks carry only what Plex lacks, so Plex always wins where it has a glyph.
        math_set = expand(MATH_RANGES) - plex_have
        sup_set = expand(SUP_RANGES) - plex_have - set(TTFont(math).getBestCmap())

        web = REPO / "web/src/fonts"
        cut(math, web / "noto-sans-math-delta.woff2", math_set, "woff2")
        cut(mono, web / "noto-sans-mono-sup.woff2", sup_set, "woff2")
        site = REPO / "site/public/fonts"
        cut(plex_regular, site / "ibm-plex-mono-400.woff2", expand(PLEX_RANGES), "woff2")
        cut(PLEX / "IBMPlexMono-Medium.woff2", site / "ibm-plex-mono-500.woff2",
            expand(PLEX_RANGES), "woff2")
        shutil.copy(web / "noto-sans-math-delta.woff2", site)
        shutil.copy(web / "noto-sans-mono-sup.woff2", site)
        export = REPO / "crates/delta-export/fonts"
        cut(plex_regular, export / "IBMPlexMono-Regular.subset.ttf", expand(PLEX_RANGES), None)
        cut(math, export / "NotoSansMath-Regular.subset.ttf", math_set, None)
        cut(mono, export / "NotoSansMono-Regular.subset.ttf", sup_set, None)

        stix = stix_static(tmp)
        cut(stix, web / "stix-two-text-italic.woff2", expand(STIX_RANGES), "woff2")
        cut(stix, export / "STIXTwoText-Italic.subset.ttf", expand(STIX_RANGES), None)
        stix_widths(stix, REPO / "crates/delta-render/src/stix_widths.rs")


if __name__ == "__main__":
    main()
