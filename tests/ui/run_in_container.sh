#!/usr/bin/env bash
set -euo pipefail

repo_root="/workspace"
artifacts_dir="$repo_root/artifacts/ui"
state_dir="$artifacts_dir/state"
recording_mp4="$artifacts_dir/discord-caps-copy-paste-demo.mp4"
recording_gif="$repo_root/docs/discord-caps-copy-paste-demo.gif"

mkdir -p "$artifacts_dir" "$state_dir"
rm -f "$recording_mp4" "$recording_gif"
rm -f "$state_dir"/*
mkdir -p "${HOME:-/tmp/dccp-home}" "${CARGO_HOME:-/tmp/dccp-cargo-home}"

export DISPLAY=:99
Xvfb :99 -screen 0 1280x720x24 >/tmp/dccp-xvfb.log 2>&1 &
xvfb_pid=$!

cleanup() {
  if [[ -n "${ffmpeg_pid:-}" ]] && kill -0 "$ffmpeg_pid" 2>/dev/null; then
    kill "$ffmpeg_pid" 2>/dev/null || true
    wait "$ffmpeg_pid" 2>/dev/null || true
  fi
  if kill -0 "$xvfb_pid" 2>/dev/null; then
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

sleep 1

ffmpeg -y \
  -video_size 1280x720 \
  -framerate 12 \
  -f x11grab \
  -i :99 \
  -t 8 \
  "$recording_mp4" >/tmp/dccp-ffmpeg.log 2>&1 &
ffmpeg_pid=$!

export DCCP_FAKE_STATE_DIR="$state_dir"
export DCCP_TETHER_BIN="$repo_root/scripts/fake_tether.sh"
export DCCP_CODEX_BIN="$repo_root/scripts/fake_codex.sh"
export DCCP_TERMINAL_CANDIDATES="xterm"
export DCCP_RANDOM_SEED="7"
export DCCP_FAKE_CODEX_SLEEP="4"

cargo run --locked -- \
  --prompt "PASTE THIS DISCORD REQUEST INTO A NEW TETHERED CODEX SESSION" \
  --cwd "$repo_root" \
  --title "Discord Caps Copy Paste Demo" \
  --discovery-timeout-ms 8000 \
  --discovery-poll-ms 250

wait "$ffmpeg_pid"
unset ffmpeg_pid

ffmpeg -y \
  -i "$recording_mp4" \
  -vf "fps=8,scale=960:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" \
  "$recording_gif" >/tmp/dccp-gif.log 2>&1

test -f "$state_dir/attach.log"
grep -q -- "-p discord" "$state_dir/attach.log"
grep -q -- "PASTE THIS DISCORD REQUEST INTO A NEW TETHERED CODEX SESSION" "$state_dir/codex-invocations.log"
test -f "$recording_gif"
