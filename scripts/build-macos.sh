#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors
#
# ntfs-mac macOS 安装包构建脚本
#
# 产出：dist/ntfs-mac-<VERSION>.pkg
#
# 安装位置（载荷根目录即文件系统根目录，因此必须按最终路径组织）：
#   /Applications/ntfs-mac.app                  GUI 应用
#   /usr/local/bin/ntfs-mac                     CLI 二进制
#   /usr/local/share/ntfs-mac/LICENSE           Apache-2.0 全文
#   /usr/local/share/ntfs-mac/NOTICE            版权与归属声明（Apache-2.0 §4(d)）
#   /usr/local/share/ntfs-mac/THIRD_PARTY_LICENSES.md
#   /usr/local/share/ntfs-mac/README.md
#   /usr/local/share/ntfs-mac/install.sh        内核扩展辅助脚本
#
# 用法：
#   chmod +x scripts/build-macos.sh
#   ./scripts/build-macos.sh                # 完整构建（GUI + CLI + pkg）
#   VERSION=0.2.0 ./scripts/build-macos.sh  # 指定版本号

set -euo pipefail

VERSION="${VERSION:-0.1.5}"
BUNDLE_ID="com.kodephp.ntfs-mac"
APP_NAME="ntfs-mac"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$PROJECT_DIR/dist"
PKG_SCRIPTS="$PROJECT_DIR/scripts/pkg-scripts"

# Every intermediate artefact (the staged .app, the CLI copy, the payload
# tree, the component package) lives in a throwaway directory created by
# `mktemp -d`. Two reasons:
#
#   1. `pkgbuild --root` treats its argument as the *filesystem root*, so the
#      payload has to be organised by final path — doing that inside `dist/`
#      meant the script first had to delete the previous run's tree. Staging
#      outside makes every run start clean without deleting anything.
#   2. Interrupted runs used to leave `dist/ntfs-mac.app` behind, and the
#      next run's `cp -R` then nested the bundle inside itself.
#
# Set KEEP_STAGE=1 to keep the staging tree for inspection.
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/ntfs-mac-build.XXXXXX")"
KEEP_STAGE="${KEEP_STAGE:-0}"

cleanup_stage() {
  if [[ "$KEEP_STAGE" == "1" ]]; then
    echo "  中间文件保留在：$STAGE"
  else
    rm -rf "$STAGE"
  fi
}
trap cleanup_stage EXIT

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

# BSD `cp -R` copies *into* an existing destination directory instead of
# replacing it, so a leftover .app from a previous build used to become
# `ntfs-mac.app/ntfs-mac.app`, silently doubling the installer size. The
# staging directory is always fresh, but keep the copy explicit.
rm -rf "$STAGE/${APP_NAME}.app"
cp -R "$APP_PATH" "$STAGE/${APP_NAME}.app"
echo "✓ .app 包就绪（$(du -sh "$STAGE/${APP_NAME}.app" | awk '{print $1}')）"

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

cp "$CLI_BIN" "$STAGE/${APP_NAME}-cli-bin"
echo "✓ CLI 二进制就绪（$(du -h "$STAGE/${APP_NAME}-cli-bin" | awk '{print $1}')）"

# ---------------------------------------------------------------------------
# Phase 3: Assemble installer payload
# ---------------------------------------------------------------------------
echo ""
echo ">>> [3/5] 组装安装包载荷..."

# The payload root *is* the filesystem root: pkgbuild installs its tree
# verbatim, so every file has to be put under the directory it should end
# up in. Files copied to the payload root land in `/` — which is what the
# first releases did, scattering `ntfs-mac.app`, `LICENSE` and `README.md`
# into the filesystem root and leaving postinstall.sh pointing at a path
# that never existed.
PAYLOAD="$STAGE/pkg-payload"
SHARE_REL="usr/local/share/${APP_NAME}"
mkdir -p "$PAYLOAD/Applications" \
         "$PAYLOAD/usr/local/bin" \
         "$PAYLOAD/$SHARE_REL"

# GUI app -> /Applications/ntfs-mac.app
cp -R "$STAGE/${APP_NAME}.app" "$PAYLOAD/Applications/${APP_NAME}.app"

# CLI binary -> /usr/local/bin/ntfs-mac (installed directly; the old
# postinstall symlink is gone)
install -m 0755 "$STAGE/${APP_NAME}-cli-bin" "$PAYLOAD/usr/local/bin/${APP_NAME}"

# Licence, attribution and docs -> /usr/local/share/ntfs-mac
# `ntfs_mac_core::license::resource_path` searches exactly this directory,
# so `ntfs-mac license --full` works from an installed copy.
install -m 0644 "$PROJECT_DIR/LICENSE"                  "$PAYLOAD/$SHARE_REL/LICENSE"
install -m 0644 "$PROJECT_DIR/NOTICE"                   "$PAYLOAD/$SHARE_REL/NOTICE"
install -m 0644 "$PROJECT_DIR/THIRD_PARTY_LICENSES.md"  "$PAYLOAD/$SHARE_REL/THIRD_PARTY_LICENSES.md"
install -m 0644 "$PROJECT_DIR/README.md"                "$PAYLOAD/$SHARE_REL/README.md"
install -m 0755 "$PROJECT_DIR/scripts/install.sh"       "$PAYLOAD/$SHARE_REL/install.sh"

echo "✓ 载荷内容："
# No -maxdepth here: a nested .app sits at depth 7+, so a depth-limited
# listing hides exactly the failure it exists to reveal.
find "$PAYLOAD" -type d | sort | sed "s|$PAYLOAD|.|"
echo "--- 文件 ---"
find "$PAYLOAD" -type f | sort | sed "s|$PAYLOAD|.|"
echo "--- 文件数：$(find "$PAYLOAD" -type f | wc -l | tr -d ' ') ---"

# Fail early if a licence artefact is missing: Apache-2.0 §4(d) requires
# the NOTICE to travel with the binary.
for required in "LICENSE" "NOTICE" "THIRD_PARTY_LICENSES.md"; do
  if [[ ! -f "$PAYLOAD/$SHARE_REL/$required" ]]; then
    echo "错误：载荷缺少 $required"
    exit 1
  fi
done

# A nested .app means `cp -R` doubled the bundle. Catch it here, before
# pkgbuild has spent time compressing the whole tree.
if [[ -e "$PAYLOAD/Applications/${APP_NAME}.app/${APP_NAME}.app" ]]; then
  echo "错误：.app 被嵌套复制（$PAYLOAD/Applications/${APP_NAME}.app/${APP_NAME}.app）"
  exit 1
fi

# ---------------------------------------------------------------------------
# Phase 4: Build .pkg via pkgbuild + productbuild --distribution
# ---------------------------------------------------------------------------
echo ""
echo ">>> [4/5] 构建 .pkg 安装包..."

# Support files and the component package live in the staging tree; only the
# finished .pkg is moved into dist/.
mkdir -p "$STAGE/packages" "$STAGE/pkg-resources"

# --- welcome.html (installer intro) ---
cat > "$STAGE/pkg-resources/welcome.html" <<HTML
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
    <li><strong>LICENSE</strong> — Apache-2.0 许可证全文</li>
    <li><strong>NOTICE</strong> — 版权与第三方组件归属声明</li>
    <li><strong>THIRD_PARTY_LICENSES.md</strong> — 静态链接的第三方 crate 许可证清单</li>
    <li><strong>README.md</strong> — 使用说明文档</li>
    <li><strong>install.sh</strong> — 辅助脚本（内核扩展安装）</li>
  </ul>
  <p>许可证与文档位于 <code>/usr/local/share/ntfs-mac/</code>，可用
     <code>ntfs-mac license --full</code> 直接查看。</p>
</body>
</html>
HTML

# --- readme.html (post-install instructions) ---
cat > "$STAGE/pkg-resources/readme.html" <<HTML
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
  <p>从 <code>/Applications</code> 打开 <strong>ntfs-mac</strong>，它会常驻
     <strong>菜单栏</strong>：下拉即可看到全部 NTFS 卷，直接挂载、在 Finder 中打开、
     卸载或弹出移动硬盘。窗口内可切换中文 / English（默认中文）。</p>
  <h3>命令行</h3>
  <pre><code>ntfs-mac list        # 列出 NTFS 分区
ntfs-mac mount &lt;id&gt;  # 挂载分区（只读）
ntfs-mac rw &lt;id&gt;     # 挂载分区（读写，需内核扩展）
ntfs-mac doctor      # 检查依赖
ntfs-mac status      # 查看状态
ntfs-mac license     # 查看许可证与第三方组件归属</code></pre>
  <h3>读写访问</h3>
  <p>如需读写访问，请运行安装脚本加载内核扩展：</p>
  <pre><code>sudo bash /usr/local/share/ntfs-mac/install.sh</code></pre>
  <h3>许可证</h3>
  <p>ntfs-mac 基于 Apache License 2.0 开源。完整文本见
     <code>/usr/local/share/ntfs-mac/LICENSE</code>，第三方组件归属见
     <code>THIRD_PARTY_LICENSES.md</code>。</p>
  <p><a href="https://github.com/kodephp/rust_ntfs">项目仓库</a></p>
</body>
</html>
HTML

# --- Component .pkg (single-payload package) ---
# NOTE: pkgbuild 输出必须位于 --package-path 目录下，productbuild 才能找到组件
# --install-location 必须是 `/`，因为载荷本身已经按最终路径组织好了
# （Applications/…、usr/local/…）。
pkgbuild --root "$PAYLOAD" \
          --install-location "/" \
          --scripts "$PKG_SCRIPTS" \
          --identifier "$BUNDLE_ID" \
          --version "$VERSION" \
          "$STAGE/packages/ntfs-mac-component-${VERSION}.pkg"
echo "✓ 组件包构建完成（$(du -h "$STAGE/packages/ntfs-mac-component-${VERSION}.pkg" | awk '{print $1}')）"

# --- distribution.xml (installer metadata) ---
# NOTE: productbuild 需要完整的 choices-outline + choice + pkg-ref 三段结构，
# 否则 component 会被静默丢弃（productbuild 输出 4K 空 pkg 但仍返回成功码）。
cat > "$STAGE/distribution.xml" <<XML
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
# Built into the staging tree, then moved into place. `mv -f` replaces an
# existing release without a separate delete step.
productbuild --distribution "$STAGE/distribution.xml" \
             --resources "$STAGE/pkg-resources" \
             --scripts "$PKG_SCRIPTS" \
             --package-path "$STAGE/packages" \
             --identifier "$BUNDLE_ID" \
             --version "$VERSION" \
             "$STAGE/${APP_NAME}-${VERSION}.pkg"

if [[ ! -f "$STAGE/${APP_NAME}-${VERSION}.pkg" ]]; then
  echo "错误：最终 .pkg 未生成"
  exit 1
fi

mkdir -p "$DIST_DIR"
mv -f "$STAGE/${APP_NAME}-${VERSION}.pkg" "$DIST_DIR/${APP_NAME}-${VERSION}.pkg"

PKG_SIZE=$(du -h "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" | awk '{print $1}')
echo "✓ 安装包构建完成：${PKG_SIZE}"

# ---------------------------------------------------------------------------
# Phase 5: Verify
# ---------------------------------------------------------------------------
echo ""
echo ">>> [5/5] 验证安装包..."

echo "--- 载荷文件 ---"
PAYLOAD_FILES="$(pkgutil --payload-files "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" 2>&1 || true)"
echo "$PAYLOAD_FILES"

# Gate the install layout. The payload root is the filesystem root, so a
# stray file here means shipping junk into `/`.
for expected in \
  "./Applications/${APP_NAME}.app" \
  "./usr/local/bin/${APP_NAME}" \
  "./usr/local/share/${APP_NAME}/LICENSE" \
  "./usr/local/share/${APP_NAME}/NOTICE" \
  "./usr/local/share/${APP_NAME}/THIRD_PARTY_LICENSES.md"
do
  if ! grep -qxF "$expected" <<< "$PAYLOAD_FILES"; then
    echo "错误：安装包缺少 $expected"
    exit 1
  fi
done

for forbidden in "./LICENSE" "./NOTICE" "./README.md" "./${APP_NAME}.app" "./scripts"; do
  if grep -qxF "$forbidden" <<< "$PAYLOAD_FILES"; then
    echo "错误：$forbidden 被安装到文件系统根目录"
    exit 1
  fi
done

# Reject a self-nested bundle: 0.1.3 shipped one because `cp -R` doubled the
# .app, which doubled the installer size and put a second copy of the GUI at
# /Applications/ntfs-mac.app/ntfs-mac.app.
if grep -qF "/${APP_NAME}.app/${APP_NAME}.app" <<< "$PAYLOAD_FILES"; then
  echo "错误：安装包内的 .app 自我嵌套"
  echo "$PAYLOAD_FILES" | grep -F "/${APP_NAME}.app/${APP_NAME}.app"
  exit 1
fi
echo "✓ 载荷路径检查通过"

echo ""
echo "--- 包内容（顶层结构） ---"
xar -tf "$DIST_DIR/${APP_NAME}-${VERSION}.pkg" 2>&1 | head -8
echo "..."

# No cleanup step: every intermediate artefact lives in $STAGE and the EXIT
# trap removes that single directory. dist/ therefore only ever holds the
# finished .pkg files.

echo ""
echo "=================================================="
echo "  ✓ macOS 安装包已生成："
echo "    dist/${APP_NAME}-${VERSION}.pkg"
echo ""
echo "  ✓ 安装位置："
echo "    /Applications/ntfs-mac.app          （GUI）"
echo "    /usr/local/bin/ntfs-mac             （CLI）"
echo "    /usr/local/share/ntfs-mac/LICENSE"
echo "    /usr/local/share/ntfs-mac/NOTICE"
echo "    /usr/local/share/ntfs-mac/THIRD_PARTY_LICENSES.md"
echo "    /usr/local/share/ntfs-mac/README.md"
echo "    /usr/local/share/ntfs-mac/install.sh"
echo ""
echo "  ✓ 安装器功能："
echo "    - welcome.html（安装前介绍）"
echo "    - readme.html（安装后指引）"
echo "    - preinstall.sh（macOS 版本检查）"
echo "    - postinstall.sh（CLI 可执行权限）"
echo "=================================================="

echo ""
echo "下一步："
echo "  1. 双击 dist/${APP_NAME}-${VERSION}.pkg 测试安装"
echo "  2. 可选：签名与公证（绕过 Gatekeeper）："
echo "     codesign --deep --force --sign \"Developer ID Installer: 你的名字\" dist/${APP_NAME}-${VERSION}.pkg"
echo "     xcrun notarytool submit dist/${APP_NAME}-${VERSION}.pkg --wait"
