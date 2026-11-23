#!/usr/bin/env bash
# Betterleaks v1.10.1 を指定した bin ディレクトリへ導入する（pinned / checksum 検証済み）。
# GitHub release から tarball と checksums.txt を取得し、sha256 が一致した場合のみ展開する。
# 既存バイナリは上書きしない（同名があればバージョン付き別名、それもあれば rc=0 でスキップ）。
# 使い方: bash scripts/install-betterleaks.sh [bin-dir]
#   既定の bin-dir は /tmp/betterleaks-bin（リポジトリ外の一時領域）。
set -euo pipefail

VERSION="1.10.1"
TAG="v${VERSION}"
RELEASE_BASE="https://github.com/betterleaks/betterleaks/releases/download/${TAG}"
DEFAULT_BIN_DIR="/tmp/betterleaks-bin"

# v1.10.1 の公式 checksums.txt から取得した固定 digest（独立 pin）。
# ダウンロードした checksums.txt との二重照合でサプライチェーンを確認する。
declare -A PINNED_SHA256=(
	["betterleaks_1.10.1_linux_x64.tar.gz"]="c174b42d7aa60caf58c0c3e555c0e4bed01fe6b0d99a412bce777b5af4ce913e"
	["betterleaks_1.10.1_linux_arm64.tar.gz"]="62a63199f9f61a6b5e5a6f58f048493db20fbc8c770014f4d903a4d97886e1bf"
	["betterleaks_1.10.1_darwin_x64.tar.gz"]="1ae2e3885d161918d8ad2e911e205e73f0d9ffba52c85b38126586c52d355ef4"
	["betterleaks_1.10.1_darwin_arm64.tar.gz"]="ca91eb3b006f15478411f235b5e2712fb15cea74f6701ebadc2364efe0af1403"
)

log() {
	printf '%s\n' "$*" >&2
}

die() {
	log "ERROR: $*"
	exit 1
}

if (($# > 1)); then
	die "引数は bin ディレクトリ 1 個だけです: $0 [bin-dir]"
fi

bin_dir="${1:-$DEFAULT_BIN_DIR}"
[ -n "$bin_dir" ] || die "bin ディレクトリが空です"

os="$(uname -s)"
arch="$(uname -m)"
case "${os}:${arch}" in
Linux:x86_64)
	asset="betterleaks_${VERSION}_linux_x64.tar.gz"
	;;
Linux:aarch64 | Linux:arm64)
	asset="betterleaks_${VERSION}_linux_arm64.tar.gz"
	;;
Darwin:arm64)
	asset="betterleaks_${VERSION}_darwin_arm64.tar.gz"
	;;
Darwin:x86_64)
	asset="betterleaks_${VERSION}_darwin_x64.tar.gz"
	;;
*)
	die "unsupported platform: os=${os} arch=${arch} (linux_x64 / linux_arm64 / darwin_arm64 / darwin_x64 のみ対応)"
	;;
esac

for tool in curl tar awk; do
	command -v "$tool" >/dev/null 2>&1 || die "${tool} is required"
done

# Linux は sha256sum、macOS は shasum -a 256 を使う（Darwin は sha256sum 標準無し）。
sha_tool=""
if command -v sha256sum >/dev/null 2>&1; then
	sha_tool="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
	sha_tool="shasum"
else
	die "sha256sum または shasum -a 256 が必要です"
fi

if [ -e "$bin_dir" ] && [ ! -d "$bin_dir" ]; then
	die "bin ディレクトリ指定先がディレクトリではありません: $bin_dir"
fi
mkdir -p "$bin_dir"

work="$(mktemp -d "${TMPDIR:-/tmp}/betterleaks-install.XXXXXX")"
cleanup() {
	if [ -n "${work:-}" ] && [ -d "$work" ]; then
		rm -rf -- "$work"
	fi
}
trap cleanup EXIT

log "Downloading ${asset} from ${RELEASE_BASE}/"
curl -fsSL --retry 3 --max-time 180 -o "$work/$asset" "${RELEASE_BASE}/${asset}" ||
	die "tarball の取得に失敗しました: ${asset}"
curl -fsSL --retry 3 --max-time 60 -o "$work/checksums.txt" "${RELEASE_BASE}/checksums.txt" ||
	die "checksums.txt の取得に失敗しました"

expected="$(awk -v name="$asset" '$2 == name { print $1; exit }' "$work/checksums.txt")"
[ -n "$expected" ] || die "checksums.txt に ${asset} のエントリがありません"
[ "${#expected}" -eq 64 ] || die "checksums.txt の ${asset} のハッシュ長が不正です: ${#expected}"
case "$expected" in
*[!0-9a-fA-F]*) die "checksums.txt の ${asset} のハッシュに 16 進以外が含まれています" ;;
esac
pinned="${PINNED_SHA256[$asset]:-}"
[ -n "$pinned" ] || die "固定 digest がありません: ${asset}"
[ "$expected" = "$pinned" ] || die "checksums.txt と固定 digest が不一致: release=${expected} pinned=${pinned}"
if [ "$sha_tool" = "sha256sum" ]; then
	actual="$(sha256sum "$work/$asset" | awk '{ print $1 }')"
else
	actual="$(shasum -a 256 "$work/$asset" | awk '{ print $1 }')"
fi
[ "$actual" = "$pinned" ] || die "sha256 不一致: expected=${pinned} actual=${actual}"
log "sha256 verified (pinned + release): ${asset}"

mkdir -p "$work/extract"
tar -xzf "$work/$asset" -C "$work/extract" || die "tarball の展開に失敗しました: ${asset}"
src="$work/extract/betterleaks"
[ -f "$src" ] || die "tarball に betterleaks バイナリが含まれていません"

dest="$bin_dir/betterleaks"
if [ -e "$dest" ]; then
	alias_dest="$bin_dir/betterleaks-${VERSION}"
	if [ -e "$alias_dest" ]; then
		log "既存の betterleaks と ${alias_dest##*/} があるため、上書きせずスキップします"
		exit 0
	fi
	dest="$alias_dest"
fi

cp -- "$src" "$dest" || die "バイナリの配置に失敗しました: $dest"
chmod 0755 "$dest" || die "実行権限の付与に失敗しました: $dest"

version_output="$("$dest" --version 2>&1)" || die "導入したバイナリが --version で失敗しました: $dest"
case "$version_output" in
*"$VERSION"*) ;;
*) die "導入したバイナリのバージョンが一致しません: $version_output" ;;
esac
log "Installed: $dest (${version_output})"
