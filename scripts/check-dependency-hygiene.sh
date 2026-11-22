#!/usr/bin/env bash
# serde_yaml / tikv-jemalloc-sys の Cargo.lock 残存を fail-closed で検査する。
#
# 判定の考え方（既定は失敗、明示例外のみ通過）:
#   1. Cargo.lock に crate が無く依存ツリーからも到達不能なら通過（消去済み）。
#   2. 依存ツリーから到達できる場合は、下記の既知例外テーブルに一致し、
#      かつ有効期限内、かつ逆依存経路が想定どおりの場合のみ通過。
#   3. それ以外（新しい必須経路・想定外の中間依存・期限切れ・lock のみの残存）は失敗。
#
# 経路は `cargo tree -i <crate> --prefix none --depth 1 --locked --offline` の直接逆依存だけを
# 検査する。例外 CHAIN の各ホップについて「直接逆依存がその 1 つだけ」である
# ことを確認するため、別経路が追加された場合は必ず失敗する（fail-closed）。
#
# 既知例外の成立条件として core/Cargo.toml の rumdl featureless 宣言も検査する
# （check-rumdl-featureless.sh に委譲）。設定が書き換わった場合は例外を適用しない。
#
# 負向テスト専用の環境変数。DEP_HYGIENE_TEST_MODE=1 のときだけ有効で、
# 通常実行時にこれらの変数が 1 つでも設定されていれば fail-closed とする。
#   DEP_HYGIENE_TEST_MODE テストモード明示フラグ（1 のみ許可）。
#   DEP_HYGIENE_TODAY    ISO date (YYYY-MM-DD)。期限判定の「今日」を差し替える。
#   DEP_HYGIENE_TREE_DIR <crate>.tree 形式の cargo tree 出力を置いたディレクトリ。
#                       ファイル欠落時は失敗（cargo 実行をスキップしない）。
#   DEP_HYGIENE_MANIFEST 検査対象の Cargo.toml パス。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LOCK_FILE="$ROOT_DIR/Cargo.lock"

die() {
	echo "ERROR: $*" >&2
	exit 1
}

# 既知例外テーブル。期限は「再レビュー期限」であり上流の対応約束ではない。
#   ID      : 例外識別子（ログ／ADR との対応に使う）
#   EXPIRY  : 有効期限（ISO date）。今日 > EXPIRY なら失敗。
#   CHAIN   : 目標 crate から rumdl までの想定ホップ（空白区切り）。
#   REASON  : 許容理由。
declare -A EXC_ID EXC_EXPIRY EXC_CHAIN EXC_REASON

EXC_ID[serde_yaml]="EXC-DEP-001"
EXC_EXPIRY[serde_yaml]="2027-03-31"
EXC_CHAIN[serde_yaml]="rumdl"
EXC_REASON[serde_yaml]="rumdl v0.3.2 の非 optional 依存。core は rumdl を default-features=false で宣言済みだが、rumdl 本体が serde_yaml をハード依存するため lock に残存する。rumdl 側の削除には依存更新が必要でスコープ外のため、期限まで明示例外とする。"

EXC_ID["tikv-jemalloc-sys"]="EXC-DEP-002"
EXC_EXPIRY["tikv-jemalloc-sys"]="2027-03-31"
EXC_CHAIN["tikv-jemalloc-sys"]="tikv-jemallocator rumdl"
EXC_REASON["tikv-jemalloc-sys"]="rumdl v0.3.2 の native target 向け非 optional 依存 tikv-jemallocator 経由（wasm ターゲットでは入らない）。core の rumdl は default-features=false だが rumdl 本体がハード依存するため lock に残存する。依存更新はスコープ外のため期限まで明示例外とする。"

TARGETS=(serde_yaml tikv-jemalloc-sys)

# テスト用 env の誤用を防ぐ。通常実行（CI / verify.sh）でテストフックが
# 設定されていた場合は「検査結果を細工できる状態」とみなし即失敗させる。
test_mode="${DEP_HYGIENE_TEST_MODE-0}"
if [[ "$test_mode" != "0" && "$test_mode" != "1" ]]; then
	die "DEP_HYGIENE_TEST_MODE は 1 のみ許可されています: ${test_mode:-<empty>}"
fi
for hook in DEP_HYGIENE_TODAY DEP_HYGIENE_TREE_DIR DEP_HYGIENE_MANIFEST; do
	if [[ -v $hook ]] && [[ "$test_mode" != "1" ]]; then
		die "テスト用環境変数 $hook が設定されています。通常実行では使用できません（DEP_HYGIENE_TEST_MODE=1 を明示してください）"
	fi
done

# 期限判定の「今日」を決定する。上書き値は形式と実在日付を検証し、
# 不正値は既定値へ戻さず失敗させる（fail-closed）。
if [[ "$test_mode" == "1" && -v DEP_HYGIENE_TODAY ]]; then
	today="$DEP_HYGIENE_TODAY"
else
	today="$(date +%F)" || die "現在日付を取得できません"
fi
[[ "$today" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || die "日付形式が不正です: ${today:-<empty>}"
normalized_today="$(date -d "$today" +%F 2>/dev/null || true)"
[[ "$normalized_today" == "$today" ]] || die "実在しない日付です: $today"
today="$normalized_today"

# 例外の前提条件: rumdl が featureless であること。
# 第一引数で差し替えられるようにし、設定改変の負向テストで使う。
if [[ "$test_mode" == "1" && -v DEP_HYGIENE_MANIFEST ]]; then
	manifest="$DEP_HYGIENE_MANIFEST"
else
	manifest="$ROOT_DIR/core/Cargo.toml"
fi
[[ -f "$manifest" ]] || die "検査対象の Cargo.toml が見つかりません: ${manifest:-<empty>}"
if ! bash "$SCRIPT_DIR/check-rumdl-featureless.sh" "$manifest"; then
	die "rumdl の featureless 宣言が壊れており依存例外を適用できません: $manifest"
fi

[[ -f "$LOCK_FILE" ]] || die "Cargo.lock が見つかりません: $LOCK_FILE"

lock_has_crate() {
	local crate="$1" rc=0
	grep -q -F "name = \"$crate\"" "$LOCK_FILE" || rc=$?
	if ((rc == 0)); then
		return 0
	fi
	if ((rc == 1)); then
		return 1
	fi
	die "Cargo.lock の検査に失敗しました (crate=$crate, grep rc=$rc)"
}

# cargo tree の出力を TREE_OUT に格納する。
# 戻り値: 0=取得成功 / 1=検査不能 / 2= crate が依存ツリーに存在しない。
TREE_OUT=""
fetch_tree() {
	local crate="$1" rc=0 err_file

	if [[ "$test_mode" == "1" && -v DEP_HYGIENE_TREE_DIR ]]; then
		local fixture="$DEP_HYGIENE_TREE_DIR/$crate.tree"
		[[ -f "$fixture" ]] || {
			echo "ERROR: 負向テスト用 fixture が見つかりません: $fixture" >&2
			return 1
		}
		TREE_OUT="$(cat "$fixture")" || return 1
		return 0
	fi

	err_file="$(mktemp)" || {
		echo "ERROR: 一時ファイルを作成できません" >&2
		return 1
	}
	# --locked で lock の変更を禁止し、--offline でネットワーク取得を禁止する
	# （verify.sh の cargo check --locked と同じ前提）。
	TREE_OUT="$(cd "$ROOT_DIR" && cargo tree -i "$crate" --prefix none --depth 1 --locked --offline 2>"$err_file")" || rc=$?
	if ((rc != 0)); then
		# 「存在しない」は正規の結果（消去済み）。それ以外は検査不能として失敗。
		if [[ -z "$TREE_OUT" ]] && grep -q 'did not match any packages' "$err_file"; then
			rm -f "$err_file"
			return 2
		fi
		echo "ERROR: cargo tree -i $crate が失敗しました (rc=$rc)" >&2
		cat "$err_file" >&2
		rm -f "$err_file"
		return 1
	fi
	rm -f "$err_file"
	return 0
}

# TREE_OUT の直接逆依存が expected の 1 件だけであることを検査する。
# 先頭行は照会した crate 自身なので無視する。
verify_hop() {
	local current="$1" expected="$2" line name count=0 matched=0
	while IFS= read -r line; do
		[[ -n "$line" ]] || continue
		name="${line%% *}"
		if [[ "$name" == "$current" ]]; then
			continue
		fi
		count=$((count + 1))
		if [[ "$name" == "$expected" ]]; then
			matched=$((matched + 1))
		else
			echo "  想定外の逆依存: $line" >&2
		fi
	done <<<"$TREE_OUT"
	if ((count != 1 || matched != 1)); then
		echo "  $current の直接逆依存は $expected のみである必要があります（観測: $count 件 / 一致: $matched 件）" >&2
		return 1
	fi
	return 0
}

check_crate() {
	local crate="$1" in_lock=0 fetch_rc=0 hop_fetch_rc=0 id expiry current hop
	local -a hops=()

	lock_has_crate "$crate" || in_lock=$?
	fetch_tree "$crate" || fetch_rc=$?

	if ((fetch_rc == 2)); then
		if ((in_lock == 0)); then
			echo "ERROR: $crate は Cargo.lock に残存していますが依存ツリーから到達できません（未使用依存の残存）" >&2
			return 1
		fi
		echo "OK: $crate は Cargo.lock / 依存ツリーの両方に存在しません（例外不要）"
		return 0
	fi
	if ((fetch_rc != 0)); then
		return 1
	fi
	if ((in_lock != 0)); then
		echo "ERROR: $crate は依存ツリーに存在しますが Cargo.lock にありません（判定不能）" >&2
		return 1
	fi

	id="${EXC_ID[$crate]:-}"
	expiry="${EXC_EXPIRY[$crate]:-}"
	if [[ -z "$id" || -z "$expiry" ]]; then
		echo "ERROR: $crate の既知例外定義がありません（fail-closed）" >&2
		return 1
	fi
	if [[ "$today" > "$expiry" ]]; then
		echo "ERROR: 既知例外 $id ($crate) は期限切れです: today=$today, expiry=$expiry" >&2
		return 1
	fi

	read -r -a hops <<<"${EXC_CHAIN[$crate]:-}"
	if ((${#hops[@]} == 0)); then
		echo "ERROR: 既知例外 $id ($crate) の許容経路が定義されていません" >&2
		return 1
	fi

	current="$crate"
	for hop in "${hops[@]}"; do
		# ホップを進めたら、その crate の直接逆依存を取り直す。
		hop_fetch_rc=0
		fetch_tree "$current" || hop_fetch_rc=$?
		if ((hop_fetch_rc != 0)); then
			echo "ERROR: 経路途中の $current の依存ツリーを取得できません (rc=$hop_fetch_rc)" >&2
			return 1
		fi
		if ! verify_hop "$current" "$hop"; then
			echo "ERROR: $crate に既知例外 $id 以外の必須依存経路があります (from=$current, expected=$hop)" >&2
			return 1
		fi
		current="$hop"
	done

	echo "OK: $crate は既知例外 $id として許容 (expiry=$expiry, chain=${EXC_CHAIN[$crate]})"
	echo "    理由: ${EXC_REASON[$crate]}"
	return 0
}

fail=0
for crate in "${TARGETS[@]}"; do
	if ! check_crate "$crate"; then
		fail=1
	fi
done

if ((fail != 0)); then
	die "依存 hygiene ゲートに失敗しました"
fi

echo "依存 hygiene ゲート成功"
