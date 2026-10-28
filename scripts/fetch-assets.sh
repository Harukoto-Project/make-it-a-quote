#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$ROOT/assets/fonts"
TWEMOJI_DIR="$ROOT/assets/twemoji/svg"
BASE="https://github.com/google/fonts/raw/main/ofl"
mkdir -p "$FONT_DIR" "$TWEMOJI_DIR"

download() {
  local url="$1" out="$2"
  if [[ -f "$out" ]]; then
    echo "skip $(basename "$out")"
    return 0
  fi
  echo "get $(basename "$out")"
  curl -fsSL "$url" -o "$out.tmp"
  mv "$out.tmp" "$out"
}

# Primary + selectable fonts
download "$BASE/mplusrounded1c/MPLUSRounded1c-Regular.ttf" "$FONT_DIR/MPLUSRounded1c-Regular.ttf"
download "$BASE/mplusrounded1c/MPLUSRounded1c-Bold.ttf" "$FONT_DIR/MPLUSRounded1c-Bold.ttf"
download "$BASE/delagothicone/DelaGothicOne-Regular.ttf" "$FONT_DIR/DelaGothicOne-Regular.ttf"
download "$BASE/dotgothic16/DotGothic16-Regular.ttf" "$FONT_DIR/DotGothic16-Regular.ttf"
download "$BASE/hachimarupop/HachiMaruPop-Regular.ttf" "$FONT_DIR/HachiMaruPop-Regular.ttf"
download "$BASE/rampartone/RampartOne-Regular.ttf" "$FONT_DIR/RampartOne-Regular.ttf"
download "$BASE/reggaeone/ReggaeOne-Regular.ttf" "$FONT_DIR/ReggaeOne-Regular.ttf"
download "$BASE/rocknrollone/RocknRollOne-Regular.ttf" "$FONT_DIR/RocknRollOne-Regular.ttf"
download "$BASE/zenoldmincho/ZenOldMincho-Regular.ttf" "$FONT_DIR/ZenOldMincho-Regular.ttf"
download "$BASE/zenoldmincho/ZenOldMincho-Bold.ttf" "$FONT_DIR/ZenOldMincho-Bold.ttf"
download "$BASE/yujisyuku/YujiSyuku-Regular.ttf" "$FONT_DIR/YujiSyuku-Regular.ttf"
download "$BASE/yuseimagic/YuseiMagic-Regular.ttf" "$FONT_DIR/YuseiMagic-Regular.ttf"
download "$BASE/brunoacesc/BrunoAceSC-Regular.ttf" "$FONT_DIR/BrunoAceSC-Regular.ttf"
download "$BASE/castorotitling/CastoroTitling-Regular.ttf" "$FONT_DIR/CastoroTitling-Regular.ttf"
download "$BASE/vinasans/VinaSans-Regular.ttf" "$FONT_DIR/VinaSans-Regular.ttf"

# Variable fonts (instance at runtime via wght)
download "$BASE/exo2/Exo2%5Bwght%5D.ttf" "$FONT_DIR/Exo2-wght.ttf"
download "$BASE/inconsolata/Inconsolata%5Bwdth%2Cwght%5D.ttf" "$FONT_DIR/Inconsolata-wdth-wght.ttf"
download "$BASE/poltawskinowy/PoltawskiNowy%5Bwght%5D.ttf" "$FONT_DIR/PoltawskiNowy-wght.ttf"
download "$BASE/dancingscript/DancingScript%5Bwght%5D.ttf" "$FONT_DIR/DancingScript-wght.ttf"

# Fallback chain (Noto)
download "$BASE/notosansjp/NotoSansJP%5Bwght%5D.ttf" "$FONT_DIR/NotoSansJP-wght.ttf"
download "$BASE/notosanssc/NotoSansSC%5Bwght%5D.ttf" "$FONT_DIR/NotoSansSC-wght.ttf"
download "$BASE/notosanstc/NotoSansTC%5Bwght%5D.ttf" "$FONT_DIR/NotoSansTC-wght.ttf"
download "$BASE/notosanskr/NotoSansKR%5Bwght%5D.ttf" "$FONT_DIR/NotoSansKR-wght.ttf"
download "$BASE/notosans/NotoSans%5Bwdth%2Cwght%5D.ttf" "$FONT_DIR/NotoSans-wdth-wght.ttf"
download "$BASE/notosansarabic/NotoSansArabic%5Bwdth%2Cwght%5D.ttf" "$FONT_DIR/NotoSansArabic-wdth-wght.ttf"
download "$BASE/notosanshebrew/NotoSansHebrew%5Bwdth%2Cwght%5D.ttf" "$FONT_DIR/NotoSansHebrew-wdth-wght.ttf"
download "$BASE/notosansthai/NotoSansThai%5Bwdth%2Cwght%5D.ttf" "$FONT_DIR/NotoSansThai-wdth-wght.ttf"
download "$BASE/notosansdevanagari/NotoSansDevanagari%5Bwdth%2Cwght%5D.ttf" "$FONT_DIR/NotoSansDevanagari-wdth-wght.ttf"
download "$BASE/notosansmath/NotoSansMath-Regular.ttf" "$FONT_DIR/NotoSansMath-Regular.ttf"
download "$BASE/notosanssymbols/NotoSansSymbols%5Bwght%5D.ttf" "$FONT_DIR/NotoSansSymbols-wght.ttf"
download "$BASE/notosanssymbols2/NotoSansSymbols2-Regular.ttf" "$FONT_DIR/NotoSansSymbols2-Regular.ttf"

# Twemoji SVG (jdecked/twemoji) — shallow clone of assets only via archive
TWEMOJI_VER="15.1.0"
TWEMOJI_MARK="$ROOT/assets/twemoji/.version"
if [[ ! -f "$TWEMOJI_MARK" ]] || [[ "$(cat "$TWEMOJI_MARK")" != "v$TWEMOJI_VER" ]]; then
  echo "fetch twemoji v$TWEMOJI_VER svg"
  TMP="$(mktemp -d)"
  curl -fsSL "https://github.com/jdecked/twemoji/archive/refs/tags/v$TWEMOJI_VER.tar.gz" \
    | tar -xz -C "$TMP" "twemoji-$TWEMOJI_VER/assets/svg"
  rm -rf "$TWEMOJI_DIR"
  mkdir -p "$(dirname "$TWEMOJI_DIR")"
  mv "$TMP/twemoji-$TWEMOJI_VER/assets/svg" "$TWEMOJI_DIR"
  echo "v$TWEMOJI_VER" > "$TWEMOJI_MARK"
  rm -rf "$TMP"
fi

echo "assets ready under $ROOT/assets"
