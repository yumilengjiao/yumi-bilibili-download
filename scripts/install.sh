#!/bin/sh
set -e

REPO="yumilengjiao/yumi-bilibili-download"
BIN_NAME="yumi-bilibili-download"
INSTALL_NAME="ybd"
INSTALL_DIR="${HOME}/.local/bin"

detect_target() {
  OS=$(uname -s)
  ARCH=$(uname -m)
  case "$OS" in
  Linux)
    case "$ARCH" in
    x86_64) echo "x86_64-unknown-linux-gnu" ;;
    *)
      echo "不支持的架构: $ARCH，目前只提供 x86_64 Linux 版本" >&2
      exit 1
      ;;
    esac
    ;;
  *)
    echo "不支持的操作系统: $OS，请使用 Windows 的 install.ps1" >&2
    exit 1
    ;;
  esac
}

get_latest_version() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSLI "https://github.com/${REPO}/releases/latest" |
      grep -i "^location:" |
      sed 's/.*\/tag\///' |
      tr -d '\r'
  elif command -v wget >/dev/null 2>&1; then
    wget -q --server-response --spider "https://github.com/${REPO}/releases/latest" 2>&1 |
      grep -i "location:" |
      sed 's/.*\/tag\///' |
      tr -d '\r'
  else
    echo "需要 curl 或 wget" >&2
    exit 1
  fi
}

download() {
  URL=$1
  OUTPUT=$2
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$OUTPUT"
  else
    wget -qO "$OUTPUT" "$URL"
  fi
}

TARGET=$(detect_target)
VERSION=$(get_latest_version)

if [ -z "$VERSION" ]; then
  echo "无法获取最新版本号" >&2
  exit 1
fi

FILE_NAME="${BIN_NAME}-${VERSION}-${TARGET}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILE_NAME}"
TMP_FILE=$(mktemp)

echo "版本: ${VERSION}"
echo "平台: ${TARGET}"
echo "正在下载..."

download "$DOWNLOAD_URL" "$TMP_FILE"
chmod +x "$TMP_FILE"
mkdir -p "$INSTALL_DIR"
mv "$TMP_FILE" "${INSTALL_DIR}/${INSTALL_NAME}"

echo "安装完成: ${INSTALL_DIR}/${INSTALL_NAME}"

case ":${PATH}:" in
*":${INSTALL_DIR}:"*) ;;
*)
  echo "提示: ${INSTALL_DIR} 不在 PATH 中，请将以下内容加入 ~/.zshrc 或 ~/.bashrc："
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  ;;
esac

echo "运行 '${INSTALL_NAME} --version' 验证安装"
