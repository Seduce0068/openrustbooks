#!/usr/bin/env bash
# WebdriverIO で実 Tauri WebView の settings 画面を検証する（fail-closed）。
# 既存 CDP E2E (tools/verify-web-reader.py) の代替ではなく、別ゲートとして実行する。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_PORT=1420
TRUNK_PID=""

cleanup() {
	if [[ -n "$TRUNK_PID" ]] && kill -0 "$TRUNK_PID" 2>/dev/null; then
		kill "$TRUNK_PID" 2>/dev/null || true
		wait "$TRUNK_PID" 2>/dev/null || true
	fi
	# trunk が子プロセスを残した場合に備え、port 1420 のlistenerも明示的に停止する。
	local pids
	pids=$(ss -ltnp 2>/dev/null | grep ":$WEB_PORT" | grep -o 'pid=[0-9]*' | cut -d= -f2 | sort -u || true)
	if [[ -n "$pids" ]]; then
		# shellcheck disable=SC2086
		kill $pids 2>/dev/null || true
	fi
}
trap cleanup EXIT

for command in cargo trunk curl npm node; do
	if ! command -v "$command" >/dev/null 2>&1; then
		echo "wdio: required command not found: $command" >&2
		exit 1
	fi
done

cd "$ROOT"

echo "wdio: building debug desktop binary with wdio feature"
cargo build -p openrustbooks-desktop --features wdio

echo "wdio: starting trunk dev server on port $WEB_PORT"
(
	cd web
	exec trunk serve --port "$WEB_PORT" >/tmp/openrustbooks-wdio-trunk.log 2>&1
) &
TRUNK_PID=$!

ready=0
for _ in $(seq 1 60); do
	if curl --max-time 5 -fsS -o /dev/null "http://localhost:$WEB_PORT/"; then
		ready=1
		break
	fi
	if ! kill -0 "$TRUNK_PID" 2>/dev/null; then
		echo "wdio: trunk serve terminated before becoming ready" >&2
		exit 1
	fi
	sleep 1
done
if [[ "$ready" -ne 1 ]]; then
	echo "wdio: trunk serve did not become ready" >&2
	exit 1
fi

export GDK_BACKEND="${GDK_BACKEND:-x11}"
if [[ -z "${DISPLAY:-}" ]]; then
	echo "wdio: DISPLAY must be set (e.g. :1 or an Xvfb display)" >&2
	exit 1
fi

echo "wdio: running WebdriverIO specs"
npm run test:wdio
