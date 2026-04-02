#!/usr/bin/env bash
set -euo pipefail

state_dir="${DCCP_FAKE_STATE_DIR:?DCCP_FAKE_STATE_DIR is required}"
mkdir -p "$state_dir"

command_name="${1:-}"
shift || true

case "$command_name" in
  status)
    printf 'fake tether healthy\n'
    ;;
  start)
    printf '%s\n' "$(date -Is)" >>"$state_dir/tether-start.log"
    printf 'fake tether started\n'
    ;;
  list)
    external=false
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --external)
          external=true
          shift
          ;;
        --directory|-d|--runner-type|-r|--state|-s)
          shift 2
          ;;
        *)
          printf 'unsupported fake tether list invocation\n' >&2
          exit 1
          ;;
      esac
    done
    if [[ "$external" == true ]]; then
      if [[ -f "$state_dir/external-sessions.txt" ]]; then
        cat "$state_dir/external-sessions.txt"
      else
        printf 'No external sessions found for runner codex\n'
      fi
    else
      printf 'unsupported fake tether list invocation\n' >&2
      exit 1
    fi
    ;;
  attach)
    attach_args=("$@")
    session_id=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --directory|-d|--runner-type|-r|--platform|-p)
          shift 2
          ;;
        *)
          if [[ -z "$session_id" ]]; then
            session_id="$1"
            shift
          else
            shift
          fi
          ;;
      esac
    done
    printf '%s\t%s\t%s\n' "$(date -Is)" "$session_id" "${attach_args[*]}" >>"$state_dir/attach.log"
    printf 'attached %s\n' "$session_id"
    ;;
  sync)
    printf '%s\t%s\n' "$(date -Is)" "${1:-}" >>"$state_dir/sync.log"
    printf 'synced %s\n' "${1:-}"
    ;;
  *)
    printf 'unsupported fake tether command: %s\n' "$command_name" >&2
    exit 1
    ;;
esac
