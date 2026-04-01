#!/usr/bin/env bash
set -euo pipefail

state_dir="${DCCP_FAKE_STATE_DIR:?DCCP_FAKE_STATE_DIR is required}"
mkdir -p "$state_dir"

session_id="${DCCP_FAKE_SESSION_ID:-codex-demo-$$}"
prompt="${1:-}"

printf '%s runner %s\n' "$session_id" "$PWD" >"$state_dir/external-sessions.txt"
printf '%s\t%s\n' "$(date -Is)" "$prompt" >>"$state_dir/codex-invocations.log"
printf '%s\n' "$session_id" >"$state_dir/last-session-id.txt"

clear
printf 'fake codex session: %s\n\n' "$session_id"
printf 'prompt received by fake codex:\n'
printf '%s\n\n' "$prompt"
printf 'selected terminal: %s\n' "${DCCP_SELECTED_TERMINAL:-unknown}"
printf 'prompt source: %s\n' "${DCCP_PROMPT_SOURCE:-unknown}"
printf '\nSleeping so the UI test can capture the terminal window...\n'

sleep "${DCCP_FAKE_CODEX_SLEEP:-4}"
