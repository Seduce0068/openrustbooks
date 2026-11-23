#!/usr/bin/env bash
# Android 向け Tauri ビルド（ADR-0352 / docs/runbook-mobile.md）。
# ローカル導入済みの JDK / Android SDK を各環境変数と PATH へ fail-closed で補う。
set -euo pipefail

usage() {
	printf 'usage: %s [debug|release]\n' "${0##*/}" >&2
}

die() {
	printf 'ERROR: %s\n' "$*" >&2
	exit 1
}

if (($# > 1)); then
	usage
	exit 2
fi

mode="${1:-release}"
case "$mode" in
debug | release) ;;
*)
	usage
	exit 2
	;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

tools_home="${DSH_ANDROID_TOOLS_HOME:-$HOME/.local/share/dsh-android-tools}"
android_home="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/.local/share/android-sdk}}"

# 既存の JAVA_HOME が無効な場合のみ、ローカル導入先の JDK を一意に特定する。
if [[ -z "${JAVA_HOME:-}" || ! -x "$JAVA_HOME/bin/java" ]]; then
	shopt -s nullglob
	local_jdks=("$tools_home"/jdk-*)
	shopt -u nullglob
	((${#local_jdks[@]} == 1)) ||
		die "JDK を一意に特定できません: $tools_home/jdk-* (JAVA_HOME を明示してください)"
	export JAVA_HOME="${local_jdks[0]}"
fi
[[ -x "$JAVA_HOME/bin/java" ]] || die "java が実行できません: $JAVA_HOME/bin/java"

[[ -d "$android_home" ]] || die "Android SDK が見つかりません: $android_home"
export ANDROID_HOME="$android_home"
export ANDROID_SDK_ROOT="$android_home"
[[ -x "$android_home/cmdline-tools/latest/bin/sdkmanager" ]] ||
	die "cmdline-tools が見つかりません: $android_home/cmdline-tools/latest/bin/sdkmanager"
[[ -d "$android_home/platform-tools" ]] ||
	die "platform-tools が見つかりません: $android_home/platform-tools"

# NDK_HOME が無効な場合、インストール済み NDK の末尾を選択する（Tauri CLI と同じ規則）。
if [[ -z "${NDK_HOME:-}" || ! -d "$NDK_HOME" ]]; then
	shopt -s nullglob
	local_ndks=("$android_home"/ndk/*)
	shopt -u nullglob
	((${#local_ndks[@]} > 0)) || die "NDK が見つかりません: $android_home/ndk"
	export NDK_HOME="${local_ndks[-1]}"
fi
[[ -f "$NDK_HOME/source.properties" ]] || die "NDK が不完全です: $NDK_HOME"

export PATH="$JAVA_HOME/bin:$android_home/platform-tools:$android_home/cmdline-tools/latest/bin:$PATH"

[[ -f desktop/gen/android/gradlew ]] ||
	die "Android プロジェクトが未生成です: cargo tauri android init を実行してください"
[[ -f web/dist/index.html ]] ||
	die "検証済み web/dist が見つかりません: beforeBuildCommand 無効化のため必要です"

# 成功実績のある構成を既定にする。既存 web/dist を使い、ABI は aarch64 のみ。
build_args=(android build --ci --target aarch64)
build_args+=(-c '{"build":{"beforeBuildCommand":"true"}}')
if [[ "$mode" == "debug" ]]; then
	build_args+=(--debug)
fi

printf 'Android build (%s): JAVA_HOME=%s ANDROID_HOME=%s NDK_HOME=%s\n' \
	"$mode" "$JAVA_HOME" "$ANDROID_HOME" "$NDK_HOME"
printf '既存 web/dist を使用し、beforeBuildCommand は設定マージで無効化します。\n'
printf 'ビルド対象は aarch64 のみ（universal 成果物でも全 ABI ではありません）。\n'
cargo tauri "${build_args[@]}"
