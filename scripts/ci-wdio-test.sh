#!/usr/bin/env bash
# WebdriverIO で実 Tauri WebView の settings 画面を検証する（fail-closed）。
# 既存 CDP E2E (tools/verify-web-reader.py) の代替ではなく、別ゲートとして実行する。
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_PORT=1420
TMP_DIR=""
TRUNK_LOG=""
TRUNK_PGID_FILE=""
TRUNK_PGID=""
TRUNK_LAUNCHER_PID=""

cleanup() {
	local rc=$?
	# 失敗時のみ trunk ログを出力する（成功時は黙って削除）。
	if ((rc != 0)) && [[ -n "$TRUNK_LOG" && -s "$TRUNK_LOG" ]]; then
		echo "wdio: trunk log ($TRUNK_LOG):" >&2
		cat "$TRUNK_LOG" >&2 || true
	fi
	# 自 trunk のプロセスグループのみ停止する。
	# port 1420 の listener を無差別に kill しない（他人のサーバーを止めない）。
	if [[ -n "$TRUNK_PGID" ]]; then
		kill -TERM -- "-$TRUNK_PGID" 2>/dev/null || true
		sleep 1
		if kill -0 -- "-$TRUNK_PGID" 2>/dev/null; then
			kill -KILL -- "-$TRUNK_PGID" 2>/dev/null || true
		fi
	fi
	# PGID が取得できなかった場合も、起動した launcher 自体は必ず始末する。
	if [[ -n "$TRUNK_LAUNCHER_PID" ]] && kill -0 "$TRUNK_LAUNCHER_PID" 2>/dev/null; then
		kill -TERM "$TRUNK_LAUNCHER_PID" 2>/dev/null || true
		sleep 0.2
		if kill -0 "$TRUNK_LAUNCHER_PID" 2>/dev/null; then
			kill -KILL "$TRUNK_LAUNCHER_PID" 2>/dev/null || true
		fi
	fi
	if [[ -n "$TRUNK_LAUNCHER_PID" ]]; then
		wait "$TRUNK_LAUNCHER_PID" 2>/dev/null || true
	fi
	if [[ -n "$TMP_DIR" ]]; then
		rm -rf "$TMP_DIR"
	fi
	return "$rc"
}
trap cleanup EXIT

for command in cargo trunk curl npm node ss setsid git ps; do
	if ! command -v "$command" >/dev/null 2>&1; then
		echo "wdio: required command not found: $command" >&2
		exit 1
	fi
done

cd "$ROOT"

# git diff HEAD 基準でステージ済み差分も検知する（作業ツリーのみの検査では漏れる）。
check_lock_clean() {
	local label="$1"
	shift
	if ! git diff --exit-code HEAD -- "$@" >/dev/null; then
		echo "wdio: $label has uncommitted changes vs HEAD" >&2
		git diff HEAD -- "$@" >&2 || true
		return 1
	fi
}

check_port_free() {
	local phase="$1"
	local listeners
	listeners="$(ss -H -ltn "sport = :$WEB_PORT")" || {
		echo "wdio: failed to inspect port $WEB_PORT ($phase)" >&2
		return 1
	}
	if [[ -n "$listeners" ]]; then
		echo "wdio: port $WEB_PORT is already in use ($phase); refusing to start trunk" >&2
		echo "$listeners" >&2
		return 1
	fi
}

# port 1420 の listener が自 PGID に属することを確認する。
# 他人のサーバーが 200 を返しても ready と判定しないための fail-closed チェック。
listener_belongs_to_trunk() {
	local pids pid pgid
	[[ -n "$TRUNK_PGID" ]] || return 1
	pids="$(ss -H -ltnp "sport = :$WEB_PORT" 2>/dev/null | grep -o 'pid=[0-9]*' | cut -d= -f2 | sort -u || true)"
	[[ -n "$pids" ]] || return 1
	while IFS= read -r pid; do
		[[ -n "$pid" ]] || continue
		pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)"
		[[ -n "$pgid" && "$pgid" == "$TRUNK_PGID" ]] || return 1
	done <<<"$pids"
	return 0
}

# ビルド前に lock ファイルがクリーンであることを確認する（既存差分は fail-closed）。
check_lock_clean "Cargo.lock" Cargo.lock || exit 1
check_lock_clean "package-lock.json" package-lock.json || exit 1

# npm ci 済みを前提に、依存ツリーの整合を検査する（npm ci 漏れを fail-closed で検知）。
echo "wdio: checking npm dependency tree (npm ls --depth=0)"
if ! npm ls --depth=0; then
	echo "wdio: npm dependencies are inconsistent; run 'npm ci' first" >&2
	exit 1
fi

# 起動前に port が空きかを確認する（ビルド前の早期失敗用）。
check_port_free "before build" || exit 1

echo "wdio: building debug desktop binary with wdio feature (--locked)"
cargo build -p openrustbooks-desktop --features wdio --locked

# --locked ビルドが lock ファイルを書き換えた場合は想定外のため fail-closed とする。
check_lock_clean "Cargo.lock (after build)" Cargo.lock || exit 1
check_lock_clean "package-lock.json (after build)" package-lock.json || exit 1

# trunk 起動直前に再検査する（ビルド中に他プロセスが port を占有した場合の保護）。
check_port_free "before trunk start" || exit 1

TMP_DIR="$(mktemp -d)"
TRUNK_LOG="$TMP_DIR/trunk.log"
TRUNK_PGID_FILE="$TMP_DIR/trunk.pgid"

echo "wdio: starting trunk dev server on port $WEB_PORT"
# setsid で独立したプロセスグループを作り、その PGID をファイルで受け取る。
# 「$$」は exec 後も変わらないため、trunk 本体の PGID として使える。
# 子 bash 内で展開させるため、スクリプト本体はシングルクォートで渡す（SC2016 は意図的）。
# shellcheck disable=SC2016
setsid bash -c '
	echo "$$" > "$1"
	cd "$2"
	exec trunk serve --port "$3"
' _ "$TRUNK_PGID_FILE" "$ROOT/web" "$WEB_PORT" >"$TRUNK_LOG" 2>&1 &
TRUNK_LAUNCHER_PID=$!

# PGID ファイルが作られるまで待つ（setsid が fork する環境でも安全に扱うため、
# ここでは launcher の生存ではなくファイルの作成だけを見る）。
for _ in $(seq 1 50); do
	if [[ -s "$TRUNK_PGID_FILE" ]]; then
		TRUNK_PGID="$(tr -dc '0-9' <"$TRUNK_PGID_FILE")"
		break
	fi
	sleep 0.1
done
if [[ -z "$TRUNK_PGID" ]]; then
	echo "wdio: could not determine trunk process group" >&2
	exit 1
fi

ready=0
for _ in $(seq 1 60); do
	# curl が 200 を返しても、listener が自 PGID でなければ ready としない。
	if curl --max-time 5 -fsS -o /dev/null "http://localhost:$WEB_PORT/" &&
		listener_belongs_to_trunk; then
		ready=1
		break
	fi
	if ! kill -0 -- "-$TRUNK_PGID" 2>/dev/null; then
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
