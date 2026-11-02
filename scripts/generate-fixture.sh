#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=1920x1080:rate=60 -frames:v 120 -c:v libx264 -preset veryfast -profile:v high -level:v 4.2 -pix_fmt yuv420p -b:v 4M -maxrate 6M -bufsize 6M -x264-params aud=1:repeat-headers=1:keyint=60:min-keyint=60:scenecut=0:bframes=0 -f h264 -y packaging/assets/test.h264
