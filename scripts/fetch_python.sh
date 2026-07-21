#!/usr/bin/env bash
# 下载并解压官方 Windows embeddable Python 到 src-tauri/resources/python，
# 供 Windows 巡检在“目标机器未安装 Python”时零安装运行。
# 该目录已在 .gitignore 中忽略，克隆仓库后打包前请先执行本脚本。
#
# 用法:  bash scripts/fetch_python.sh
set -euo pipefail

PY_VERSION="3.13.12"
ARCH="amd64"
ZIP="python-${PY_VERSION}-embed-${ARCH}.zip"
URL="https://www.python.org/ftp/python/${PY_VERSION}/${ZIP}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${ROOT}/src-tauri/resources/python"

echo "==> 目标目录: ${DEST}"
if [ -f "${DEST}/python.exe" ]; then
  echo "==> 已存在 python.exe，跳过下载。如需重新获取请先删除该目录。"
  exit 0
fi

mkdir -p "${DEST}"
TMP="$(mktemp -d)"
echo "==> 下载 ${URL}"
curl -fL "${URL}" -o "${TMP}/${ZIP}"

echo "==> 解压到 ${DEST}"
if command -v unzip >/dev/null 2>&1; then
  unzip -o "${TMP}/${ZIP}" -d "${DEST}" >/dev/null
else
  # Windows Git Bash 无 unzip 时回退到 PowerShell
  powershell -NoProfile -Command "Expand-Archive -Force -Path '${TMP}/${ZIP}' -DestinationPath '${DEST}'"
fi

rm -rf "${TMP}"
echo "==> 完成。已就绪: ${DEST}/python.exe"
