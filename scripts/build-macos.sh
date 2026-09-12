#!/usr/bin/env bash
#
# ntfs-mac macOS 安装包构建脚本
#
# 产出：dist/ntfs-mac-<VERSION>.pkg
# 包含：
#   - ntfs-mac.app (GUI 应用，装到 /Applications)
#   - ntfs-mac (CLI 二进制，装到 /usr/local/bin，postinstall 建软链)
#   - LICENSE (Apache-2.0)
#   - README.md
#   - scripts/install.sh
#
# 用法：
#   chmod +x scripts/build-macos.sh
#   ./scripts/build-macos.sh                # 完整构建（GUI + CLI + pkg）
#   VERSION=0.2.0 ./scripts/build-macos.sh  # 指定版本号

set -euo pipefail

VERSION="${VERSION:-0.1.0}"
BUNDLE_ID="com.kodephp.ntfs-mac"
APP_NAME="ntfs-mac"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$PROJECT_DIR/dist"
PKG_SCRIPTS="$PROJECT_DIR/scripts/pkg-scripts"

cd "$PROJECT_DIR"

echo "=================================================="
echo "  ntfs-mac macOS 安装包构建"
echo "  版本    : $VERSION"
echo "  BundleID: $BUNDLE_ID"
echo "=================================================="

# ---------------------------------------------------------------------------
# Phase 0: Pre-flight
# ---------------------------------------------------------------------------
echo ""
echo ">>> [0/5] 检查前置条件..."

command -v rustc >/dev/null || {
  echo "错误：未找到 rustc。安装方式：curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
}
command -v pkgbuild >/dev/null || {
  echo "错误：未找到 pkgbuild。此脚本需要 macOS 开发者工具。"
  exit 1
}
command -v productbuild >/dev/null || {
  echo "错误：未找到 productbuild。"
  exit 1
}
command -v xar >/dev/null || {
  echo "错误：未找到 xar。"
  exit 1
}
command -v pkgutil >/dev/null || {
  echo "错误：未找到 pkgutil。"
  exit 1
}
mkdir -p "$DIST_DIR"
echo "✓ 前置条件检查通过"

# ---------------------------------------------------------------------------
# Phase 1: Build GUI .app via Tauri
# ---------------------------------------------------------------------------
echo ""
echo ">>> [1/5] 构建 GUI .app（Tauri）..."

cargo tauri build --bundles app --config crates/ntfs-mac-tauri/src-tauri/tauri.conf.json 2>&1 | tail -15

# Tauri 的 .app 输出路径取决于是否在 workspace 根运行 + 是否指定 --bundles
# 依次尝试两个常见位置
APP_PATH=""
for CANDIDATE in \
  "target/release/bundle/macos/${APP_NAME}.app" \
  "target/release/bundle/app/${APP_NAME}.app" \
  "crates/ntfs-mac-tauri/src-tauri/target/release/bundle/macos/${APP_NAME}.app" \
  "crates/ntfs-mac-tauri/src-tauri/target/release/bundle/app/${APP_NAME}.app"
do
  if [[ -d "$CANDIDATE" ]]; then
    APP_PATH="$CANDIDATE"
    break
  fi
done
if [[ -z "$APP_PATH" ]]; then
  echo "错误：.app 包未在任何预期位置找到"
  echo "  已搜索：target/release/bundle/{macos,app}/ 和 crates/ntfs-mac-tauri/src-tauri/target/release/bundle/{macos,app}/"
  exit 1
fi
echo "  找到 .app：$APP_PATH"

cp -R "$APP_PATH" "$DIST_DIR/${APP_NAME}.app"
echo "✓ .app 包就绪（$(du -sh "$DIST_DIR/${APP_NAME}.app" | awk '{print $1}')）"

# ---------------------------------------------------------------------------
# Phase 2: Build CLI release binary
# ---------------------------------------------------------------------------
echo ""
echo ">>> [2/5] 构建 CLI 二进制（release）..."

cargo build --release --package ntfs-mac-cli

CLI_BIN="target/release/${APP_NAME}"
if [[ ! -f "$CLI_BIN" ]]; then
  echo "错误：CLI 二进制未找到：$CLI_BIN"
  exit 1
fi

cp "$CLI_BIN" "$DIST_DIR/${APP_NAME}-cli-bin"
echo "✓ CLI 二进制就绪（$(du -h "$DIST_DIR/${APP_NAME}-cli-bin" | awk '{print $1}')）"

# ---------------------------------------------------------------------------
# Phase 3: Assemble installer payload
# ---------------------------------------------------------------------------
echo ""
echo ">>> [3/5] 组装安装包载荷..."

PAYLOAD="$DIST_DIR/pkg-payload"
rm -rf "$PAYLOAD"
mkdir -p "$PAYLOAD" "$PAYLOAD/scripts"

# GUI app
cp -R "$DIST_DIR/${APP_NAME}.app" "$PAYLOAD/"
# CLI binary
cp "$DIST_DIR/${APP_NAME}-cli-bin" "$PAYLOAD/${APP_NAME}-cli"
# License + docs
cp "$PROJECT_DIR/LICENSE" "$PAYLOAD/"
cp "$PROJECT_DIR/README.md" "$PAYLOAD/"
# Helper install script (kernel extension installer)
cp "$PROJECT_DIR/scripts/install.sh" "$PAYLOAD/scripts/"

echo "✓ 载荷内容："
ls -la "$PAYLOAD"

# ---------------------------------------------------------------------------
# Phase 4: Build .pkg via pkgbuild + productbuild --distribution
# ---------------------------------------------------------------------------
echo ""
echo ">>> [4/5] 构建 .pkg 安装包..."

rm -rf "$DIST_DIR/packages" "$DIST_DIR/pkg-resources"
rm -f "$DIST_DIR/distribution.xml" \
      "$DIST_DIR/${APP_NAME}-${VERSION}.pkg"
mkdir -p "$DIST_DIR/packages" "$DIST_DIR/pkg-resources"

# --- welcome.html (installer intro) ---
cat > "$DIST_DIR/pkg-resources/welcome.html" <<HTML
<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><style>
  body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 40px; color: #1d1d1f; }
  h1 { font-size: 24px; margin-bottom: 8px; }
  h3 { font-size: 16px; margin-top: 20px; margin-bottom: 8px; }
  p { font-size: 14px; color: #555; margin-bottom: 16px; }
  .badge { display: inline-block; padding: 4px 12px; background: #0071e3; color: white; border-radius: 4px; font-size: 12px; font-weight: 600; margin-bottom: 20px; }
  ul { padding-left: 20px; }
  li { margin-bottom: 6px; font-size: 14px; }
  code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: Menlo, monospace; font-size: 13px; }
</style></head>
<body>
  <div class="badge">v${VERSION}</div>
  <h1>ntfs-mac 安装程序</h1>
  <p>macOS NTFS 读写工具集——挂载、卸载、格式化、修复、复制。</p>
  <h3>安装内容</h3>
  <ul>
    <li><strong>ntfs-mac.app</strong> — 图形界面应用（安装到 <code>/Applications</code>）</li>
    <li><strong>ntfs-mac</strong> — 命令行工具（安装到 <code>/usr/local/bin</code>）</li>
    <li><strong>LICENSE</strong> — Apache-2.0 许可证</li>
    <li><strong>README.md</strong> — 使用说明文档</li>
    <li><strong>scripts/install.sh</strong> — 辅助脚本（内核扩展安装）</li>
  </ul>
</body>
</html>
HTML

# --- readme.html (post-install instructions) ---
cat > "$DIST_DIR/pkg-resources/readme.html" <<HTML
<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><style>
  body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 40px; color: #1d1d1f; }
  h1 { font-size: 24px; }
  h3 { font-size: 16px; margin-top: 20px; margin-bottom: 8px; }
  p { font-size: 14px; color: #555; margin-bottom: 12px; }
  code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: Menlo, monospace; font-size: 13px; }
  pre { background: #f4f4f4; padding: 12px; border-radius: 4px; overflow-x: auto; }
  pre code { background: none; padding: 0; }
  ul { padding-left: 20px; }
  li { margin-bottom: 6px; }
  a { color: #0071e3; }
</style></head>
<body>
  <h1>ntfs-mac 安装完成</h1>
  <p>安装已完成。以下是使用方式：</p>
  <h3>图形界面</h3>
  <p>从 <code>/Applications</code> 打开 <strong>ntfs-mac</strong>，浏览 NTFS 卷、挂载分区、复制文件、运行诊断。</p>
  <h3>命令行</h3>
  <pre><code>ntfs-mac list        # 列出 NTFS 分区
ntfs-mac mount &lt;id&gt;  # 挂载分区（只读）
ntfs-mac rw &lt;id&gt;     # 挂载分区（读写，需内核扩展）
ntfs-mac doctor      # 检查依赖
ntfs-mac status      # 查看状态
ntfs-mac sponsor     # 显示赞助二维码路径</code></pre>
  <h3>读写访问</h3>
  <p>如需读写访问，请运行安装脚本加载内核扩展：</p>
  <pre><code>sudo bash /path/to/ntfs-mac/scripts/install.sh</code></pre>
  <h3>许可证</h3>
  <p>ntfs-mac 基于 Apache License 2.0 开源。</p>
  <p><a href="https://github.com/kodephp/ntfs-mac">项目仓库</a></p>
</body>
</html>
HTML

# --- Component .pkg (single-payload package) ---
# NOTE: pkgbuild 输出必须位于 --package-path 目录下，productbuild 才能找到组件
pkgbuild --root "$PAYLOAD" \
          --scripts "$PKG_SCRIPTS" \
          --identifier "$BUNDLE_ID" \
          --version "$VERSION" \
          "$DIST_DIR/packages/ntfs-mac-component-${VERSION}.pkg"
echo "✓ 组件包构建完成（$(du -h "$DIST_DIR/packages/ntfs-mac-component-${VERSION}.pkg" | awk '{print $1}')）"

# --- distribution.xml (installer metadata) ---
# NOTE: productbuild 需要完整的 choices-outline + choice + pkg-ref 三段结构，
# 否则 component 会被静默丢弃（productbuild 输出 4K 空 pkg 但仍返回成功码）。
cat > "$DIST_DIR/distribution.xml" <<XML
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>ntfs-mac ${VERSION}</title>
    <welcome file="welcome.html" mime-type="text/html"/>
    <readme file="readme.html" mime-type="text/html"/>
    <scripts>
        <preinstall file="preinstall.sh" mime-type="text/x-sh"/>
        <postinstall file="postinstall.sh" mime-type="text/x-sh"/>
    </scripts>
    <options customize="allow" require-scripts="false" rootVolumeOnly="false"/>
    <choices-outline>
        <line choice="default">
            <line choice="${BUNDLE_ID}"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="${BUNDLE_ID}" visible="false">
        <pkg-ref id="${BUNDLE_ID}" onConclusion="none"/>
    </choice>
    <pkg-ref id="${BUNDLE_ID}" version="${VERSION}" onConclusion="none">ntfs-mac-component-${VERSION}.pkg</pkg-ref>
</installer-gui-script>
XML

# --- Final .pkg via productbuild --distribution ---
productbuild --distribution "$DIST_DIR/distribution.xml" \
             --resources "$DIST_DIR/pkg-resources" \
             --scripts "$PKG_SCRIPTS" \
             --package-path "$DIST_DIR/packages" \
             --identifier "$BUNDLE_ID" \
             --version "$VERSION" \
             "$DIST_DIR/${APP_NAME}-${VERSION}.pkg"

if [[ ! -f "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" ]]; then
  echo "错误：最终 .pkg 未生成"
  exit 1
fi

PKG_SIZE=$(du -h "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" | awk '{print $1}')
echo "✓ 安装包构建完成：${PKG_SIZE}"

# ---------------------------------------------------------------------------
# Phase 5: Verify + Cleanup
# ---------------------------------------------------------------------------
echo ""
echo ">>> [5/5] 验证安装包..."

echo "--- 载荷文件 ---"
pkgutil --payload-files "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" 2>&1 || true
echo ""
echo "--- 包内容（顶层结构） ---"
xar -tf "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" 2>&1 | head -8
echo "..."

echo ""
echo "清理中间文件..."
rm -rf "$PAYLOAD"
rm -rf "$DIST_DIR/packages"
rm -rf "$DIST_DIR/pkg-resources"
rm -f "$DIST_DIR/ntfs-mac-component-${VERSION}.pkg"
rm -f "$DIST_DIR/distribution.xml"
rm -f "$DIST_DIR/${APP_NAME}-cli-bin"
rm -rf "$DIST_DIR/${APP_NAME}.app"

echo ""
echo "=================================================="
echo "  ✓ macOS 安装包已生成："
echo "    dist/${APP_NAME}-${VERSION}.pkg"
echo ""
echo "  ✓ 安装包内容："
echo "    - ntfs-mac.app（GUI，安装到 /Applications）"
echo "    - ntfs-mac（CLI，软链到 /usr/local/bin）"
echo "    - LICENSE（Apache-2.0）"
echo "    - README.md"
echo "    - scripts/install.sh"
echo ""
echo "  ✓ 安装器功能："
echo "    - welcome.html（安装前介绍）"
echo "    - readme.html（安装后指引）"
echo "    - preinstall.sh（macOS 版本检查）"
echo "    - postinstall.sh（CLI 软链、权限设置）"
echo "=================================================="

echo ""
echo "下一步："
echo "  1. 双击 dist/${APP_NAME}-${VERSION}.pkg 测试安装"
echo "  2. 可选：签名与公证（绕过 Gatekeeper）："
echo "     codesign --deep --force --sign \"Developer ID Installer: 你的名字\" dist/${APP_NAME}-${VERSION}.pkg"
echo "     xcrun notarytool submit dist/${APP_NAME}-${VERSION}.pkg --wait"
