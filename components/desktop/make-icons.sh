#!/bin/sh
# Make reva-fork's icons from its logo, then the web favicon from those.
#
#   components/desktop/make-icons.sh            (ImageMagick `convert`, perl; python3
#                                     with Pillow and numpy for the cut-out)
#   components/desktop/make-icons.sh --no-cut   reuse icons/src/reva-fork-logo.png
#
# Source: crates/reva-desktop/icons/src/reva-fork-logo.jpg (1024x1024, the logo on a
# dark photo).  src/cutout.py cuts it out to src/reva-fork-logo.png (a
# transparent master, committed so the rest needs only ImageMagick), then:
#   icons/icon.png    256x256: Tauri's window icon and the Linux bundle icon
#                     (512 is too large for an X11 window icon: GTK drops it)
#   icons/icon.ico    16..256: Windows exe and installers
#   icons/icon.icns   16..512 at 1x/2x: the macOS .app (written by perl;
#                     ImageMagick cannot write ICNS)
# and finally scripts/make-favicon.sh (web favicon.png/.ico from icon.png).
# All outputs are committed; rerun only when the logo changes.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
icons=$here/crates/reva-desktop/icons
master=$icons/src/reva-fork-logo.png
if [ "${1:-}" != --no-cut ]; then
    python3 "$icons/src/cutout.py" "$icons/src/reva-fork-logo.jpg" "$master.tmp" 20 150
    convert "$master.tmp" -filter Lanczos -resize 512x512 -strip "PNG32:$master"
    rm -f "$master.tmp"
fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
for n in 16 24 32 48 64 128 256 512; do
    convert "$master" -filter Lanczos -resize "${n}x$n" -strip "PNG32:$tmp/$n.png"
done
cp "$tmp/256.png" "$icons/icon.png"
convert "$tmp/16.png" "$tmp/24.png" "$tmp/32.png" "$tmp/48.png" "$tmp/64.png" \
    "$tmp/128.png" "$tmp/256.png" "$icons/icon.ico"
perl -e '
    my ($dir, $out) = @ARGV;
    my @e = (["ic11", 32], ["ic12", 64], ["ic07", 128], ["ic13", 256],
             ["ic08", 256], ["ic14", 512], ["ic09", 512]);
    my $body = "";
    for (@e) {
        my ($type, $n) = @$_;
        open my $f, "<:raw", "$dir/$n.png" or die "$n.png: $!";
        local $/; my $png = <$f>;
        $body .= pack("A4N", $type, 8 + length $png) . $png;
    }
    open my $o, ">:raw", $out or die "$out: $!";
    print $o pack("A4N", "icns", 8 + length $body), $body;
' "$tmp" "$icons/icon.icns"
"$here/../../scripts/make-favicon.sh"
echo "make-icons: $icons/icon.png icon.ico icon.icns"
