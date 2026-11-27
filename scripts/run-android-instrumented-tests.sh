#!/usr/bin/env bash
# Android instrumented test（Espresso Web）をエミュレータで実行する。
# Tauri の rustBuild は dev/build セッションの options server と
# NDK リンカ設定を必要とするため、ここで前提条件を揃える。
set -euo pipefail

die() {
	printf 'ERROR: %s\n' "$*" >&2
	exit 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/.local/share/android-sdk}}"
JAVA_HOME="${JAVA_HOME:-$HOME/.local/share/dsh-android-tools/jdk-zulu-25.38.201}"
if [[ -z "${NDK_HOME:-}" ]]; then
	shopt -s nullglob
	ndks=("$ANDROID_HOME"/ndk/*)
	shopt -u nullglob
	((${#ndks[@]} > 0)) || die "NDK が見つかりません: $ANDROID_HOME/ndk"
	NDK_HOME="${ndks[-1]}"
fi
ADB="$ANDROID_HOME/platform-tools/adb"
ANDROID_GEN="$ROOT_DIR/desktop/gen/android"
OPTIONS_FILE="$ANDROID_GEN/.tauri/cli-options-server.json"

[[ -x "$ADB" ]] || die "adb が見つかりません: $ADB"
[[ -x "$JAVA_HOME/bin/java" ]] || die "java が見つかりません: $JAVA_HOME/bin/java"
[[ -n "$NDK_HOME" && -d "$NDK_HOME" ]] || die "NDK が見つかりません"
[[ -f "$ANDROID_GEN/gradlew" ]] || die "Android プロジェクトが未生成です"

# Tauri CLI の options server が必要（dev/build 実行中のみ存在）。
[[ -f "$OPTIONS_FILE" ]] ||
	die "options server がありません。cargo tauri android dev を別端末で起動してください"

# 生成プロジェクトが追跡設定と一致し、テスト sourceSets が配線されていることを確認する。
identifier="$(sed -n 's/.*"identifier": "\([^"]*\)".*/\1/p' "$ROOT_DIR/desktop/tauri.conf.json" | head -1)"
gradle_file="$ANDROID_GEN/app/build.gradle.kts"
[[ -f "$gradle_file" ]] || die "build.gradle.kts がありません: $gradle_file"
namespace="$(sed -n 's/^[[:space:]]*namespace = "\(.*\)"/\1/p' "$gradle_file" | head -1)"
[[ "$namespace" == "$identifier" ]] ||
	die "namespace ($namespace) が tauri.conf.json ($identifier) と不一致です。gen/android を再生成してください"
grep -q 'android-test/androidTest' "$gradle_file" ||
	die "androidTest sourceSets が未配線です。docs/runbook-mobile.md を参照してください"
grep -q 'androidx.test.espresso:espresso-web' "$gradle_file" ||
	die "espresso-web 依存が未配線です。docs/runbook-mobile.md を参照してください"

# rustBuild が複数デバイスで失敗するため、エミュレータ以外を切断する。
for serial in $("$ADB" devices | awk 'NR>1 && $2=="device" && $1!~/^emulator-/ {print $1}'); do
	"$ADB" disconnect "$serial" >/dev/null 2>&1 || true
done
devices="$("$ADB" devices | awk 'NR>1 && $2=="device" {print $1}' | head -1)"
[[ -n "$devices" ]] || die "接続中のエミュレータがありません"
export ANDROID_SERIAL="$devices"

# android-studio-script が NDK リンカを見つけられるよう明示する。
export JAVA_HOME ANDROID_HOME NDK_HOME
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android24-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS='-Clink-arg=-landroid -Clink-arg=-llog -Clink-arg=-lOpenSLES'

printf '端末: %s / NDK: %s\n' "$ANDROID_SERIAL" "$NDK_HOME"
cd "$ANDROID_GEN"
./gradlew :app:connectedX86_64DebugAndroidTest "$@"
