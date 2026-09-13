# ntfs-mac

macOS 上的 NTFS 工具集——挂载、卸载、格式化、修复、复制，驱动自 `ntfs-3g` + `diskutil`。

**为什么需要它？** macOS 原生只能**只读**挂载 NTFS 分区。移动硬盘、U 盘、Windows 备份盘在 Mac 上无法创建文件夹或写入。本项目通过 `ntfs-3g` + FUSE（macFUSE 或 FUSE-T）提供完整的 NTFS 读写能力。

## 架构

```
┌─────────────────────────────────────────────┐
│  ntfs-mac-cli          ntfs-mac-tauri       │
│  (clap 子命令 CLI)     (Tauri v2 + Vanilla)  │
│         └──────────┬───────────┘            │
│                    ▼                        │
│             ntfs-mac-core                   │
│  runner · deps · device · mount · daemon    │
│  format · fix · copy · config               │
│                    ▼                        │
│        ntfs-3g · diskutil · hdiutil         │
│        newfs_ntfs · ntfsfix · fsck_ntfs     │
│        rsync (copy)                         │
└─────────────────────────────────────────────┘
```

- **core**：子进程封装 + plist 解析 + 配置 + 安全确认 token，~1500 LOC，零 `unsafe`、零 async。
- **cli**：`clap` derive 子命令，支持 `-j/--json`、`-v/--verbose`、`--no-color`。
- **tauri**：深色主题单页 GUI，卷列表 + 操作按钮 + 格式化二次确认。

## 前置依赖

macOS 必须安装 `ntfs-3g` 和 FUSE 驱动，否则挂载将失败。两种方式：

**方式一：一键脚本**
```bash
./scripts/install.sh
```
自动检测架构（Apple Silicon 选 `fuse-t`，Intel 选 `macfuse`），安装 Homebrew 包并验证。

**方式二：手动安装**
```bash
# Apple Silicon
brew install --cask fuse-t
brew install ntfs-3g

# Intel Mac
brew install --cask macfuse
brew install ntfs-3g
```

> **注意**：macFUSE / FUSE-T 是内核扩展，安装后需在 **系统设置 → 隐私与安全性** 中批准，可能需要重启。

## 安装

**方式一：macOS 安装包（.pkg，推荐）**

从 [GitHub Releases](https://github.com/kodephp/rust_ntfs/releases) 下载 `ntfs-mac-<version>.pkg`，双击安装。
一次安装 GUI 应用（`/Applications/ntfs-mac.app`）、CLI（`/usr/local/bin/ntfs-mac`）
以及许可证与归属文档（`/usr/local/share/ntfs-mac/`）。

**方式二：Homebrew Cask**

cask 定义在仓库的 [`Casks/ntfs-mac.rb`](Casks/ntfs-mac.rb)，同样通过 GitHub Releases 上的 `.pkg` 安装：

```bash
brew install --cask https://raw.githubusercontent.com/kodephp/rust_ntfs/main/Casks/ntfs-mac.rb
```

卸载：

```bash
brew uninstall --cask ntfs-mac
```

**方式三：cargo install（仅 CLI）**

```bash
cargo install ntfs-mac-cli
```

**方式四：从源码构建**

见下节。

> **注意**：无论哪种方式，读写 NTFS 都需要 FUSE 驱动（`ntfs-3g` + macFUSE/FUSE-T）。
> 安装方法见上节，安装后需在 **系统设置 → 隐私与安全性** 中批准。

## 从源码构建

```bash
# 克隆
git clone https://github.com/kodephp/rust_ntfs
cd rust_ntfs

# CLI（默认 workspace 成员，快速）
cargo build --release -p ntfs-mac-cli
# → target/release/ntfs-mac

# GUI（含 Tauri，较慢）
cargo build --release -p ntfs-mac-tauri
# → target/release/ntfs-mac.app
```

**Rust 版本要求**：1.85+（edition 2024）

## CLI 用法

```
ntfs-mac [OPTIONS] <COMMAND>

全局选项:
  -c, --config <PATH>       指定配置文件路径
  -j, --json                JSON 输出
  -v, --verbose             详细输出
      --no-color            禁用彩色
      --log-level <LEVEL>   日志级别: error, warn, info, debug, trace（默认: warn）
```

### `doctor` — 检查依赖

```bash
ntfs-mac doctor
```

输出示例：
```
Platform: macOS 14.5 on aarch64

Dependencies:
  ✓ diskutil
  ✓ hdiutil
  ✗ ntfs-3g       missing — brew install ntfs-3g
  ✗ newfs_ntfs    missing — ships with `brew install ntfs-3g`
  ✗ ntfsfix       missing — ships with `brew install ntfs-3g`
  ✗ fsck_ntfs     missing — ships with `brew install ntfs-3g`
  ✓ rsync
  ✗ fuse-driver   missing — brew install --cask macfuse (Intel) OR fuse-t (Apple Silicon)

Ready: false
```

### `list` — 列出 NTFS 卷

```bash
ntfs-mac list
```

输出示例：
```
Device     Name               Size       Status       Mount Point
--------------------------------------------------------------------------------
disk2s2    MyData             931.5 GiB  mounted      /Volumes/MyData
disk3s2    USBBackup          57.2 GiB   unmounted    -
```

### `mount` / `unmount` — 挂载与卸载

```bash
# 按设备号或卷名挂载
ntfs-mac mount disk2s2
ntfs-mac mount MyData

# 挂载所有未挂载的 NTFS 卷（Mounty 式一键挂载）
ntfs-mac mount all

# 指定挂载点
ntfs-mac mount disk2s2 --mount-point /Volumes/Backup

# 只读模式（安全模式，不写磁盘）
ntfs-mac mount disk2s2 -r

# 强制使用 ntfs-3g 而非 diskutil 自动挂载
ntfs-mac mount disk2s2 --force-ntfs3g

# 卸载
ntfs-mac unmount disk2s2
```

### `rw` — 只读转读写

macOS 原生驱动将 NTFS 只读挂载。此命令自动卸载并以读写模式重新挂载：

```bash
# 按设备号或卷名切换为读写
ntfs-mac rw disk2s2
ntfs-mac rw MyData
```

### `status` — 状态总览

显示依赖状态、守护进程状态和卷列表：

```bash
ntfs-mac status
```

输出示例：
```
ntfs-mac Status
  Dependencies: READY
  Daemon: INSTALLED
  Volumes: 2
    Mounted:   1 ✓
    Unmounted: 1 ○

  ● MyData (disk2s2) — /Volumes/MyData
  ○ USBBackup (disk3s2) — -
```

### `daemon` — 自动挂载守护进程

监听磁盘热插拔，自动挂载新出现的 NTFS 卷（对标 Mounty）：

```bash
# 前台运行（Ctrl+C 停止）
ntfs-mac daemon

# 自定义轮询间隔（秒，默认 3）
ntfs-mac daemon --interval 5

# 安装为 LaunchAgent（登录自启动）
ntfs-mac daemon --install

# 查看守护进程状态
ntfs-mac daemon --status

# 卸载 LaunchAgent
ntfs-mac daemon --uninstall
```

LaunchAgent 路径：`~/Library/LaunchAgents/com.kodephp.ntfs-mac.plist`
日志路径：`~/Library/Logs/ntfs-mac.err.log`

### `format` — 格式化（破坏性）

**⚠️ 将擦除目标卷上的所有数据。**

```bash
ntfs-mac format disk2s2
ntfs-mac format disk2s2 --label "NewName"
ntfs-mac format disk2s2 --label "NewName" --quick
```

交互确认流程：
```
This will ERASE all data on the selected volume.
Volume: MyData (disk2s2)
Type 'format disk2s2 MyData' to confirm:
```

必须精确输入确认短语才能继续。

### `fix` — 修复

```bash
# 使用 ntfsfix（快速修复，macOS 上安全）
ntfs-mac fix disk2s2

# 使用 fsck_ntfs（更彻底，但耗时）
ntfs-mac fix disk2s2 --fsck
```

### `copy` — 复制

```bash
# 使用 rsync（推荐，带进度条）
ntfs-mac copy ./local_dir /Volumes/MyData/backup

# 使用 --delete 删除目标多余文件
ntfs-mac copy ./local_dir /Volumes/MyData/backup --delete

# 干跑模式
ntfs-mac copy ./local_dir /Volumes/MyData/backup --dry-run
```

### `config` — 管理配置

```bash
ntfs-mac config show     # 查看当前配置
ntfs-mac config path     # 显示配置文件路径
ntfs-mac config set mount_base /Volumes
ntfs-mac config set fuse_driver fuse-t
ntfs-mac config set require_confirmation false
ntfs-mac config reset    # 重置为默认
```

### `sponsor` — 赞助/收款二维码

显示赞助收款二维码的位置。GUI 底部和 CLI 均集成了赞助入口。

```bash
# 显示二维码路径
ntfs-mac sponsor

# 在 Finder 中显示二维码文件
ntfs-mac sponsor --reveal

# JSON 输出
ntfs-mac sponsor --json
```

**替换为自有二维码**：

1. 生成支付二维码图片（微信/支付宝收款码等，PNG/SVG/JPG 均可）
2. 替换占位文件：`crates/ntfs-mac-tauri/src/assets/sponsor-qr.svg`
3. 重新构建：`cargo build --release`

打包后二维码文件会随 `.app` 一起分发（位于 `.app/Contents/Resources/assets/`），GUI 窗口底部"Support This Project"卡片和 CLI `sponsor` 命令均可访问。

### `license` — 许可证与第三方归属

查看本项目许可证、`NOTICE` 与静态链接的第三方组件清单。所有内容都来自二进制本身
（第三方清单在编译期内嵌），因此离线或从任意位置运行都可用。

```bash
# 许可证摘要（SPDX、版权、Apache-2.0 权利义务概要、文件位置）
ntfs-mac license

# 完整 Apache-2.0 文本（从安装目录或源码树读取 LICENSE）
ntfs-mac license --full

# 内嵌的第三方 crate 清单（约 490 个 crate，标注 CLI / GUI 归属）
ntfs-mac license --third-party

# JSON 输出
ntfs-mac license --json
```

输出示例：

```
ntfs-mac 0.1.3 — Apache-2.0
Copyright 2026 kodephp contributors
https://www.apache.org/licenses/LICENSE-2.0

  permissions  commercial use, modification, distribution, patent use, private use
  conditions   license and copyright notice, state changes
  limitations  liability, trademark use, warranty

Bundled files
  LICENSE                  /usr/local/share/ntfs-mac/LICENSE
  NOTICE                   /usr/local/share/ntfs-mac/NOTICE
  THIRD_PARTY_LICENSES.md  487 crates (87 CLI, 447 GUI) — embedded in this binary
```

`LICENSE` / `NOTICE` 会在多个候选目录中查找（源码树 → `/usr/local/share/ntfs-mac`
→ Homebrew `share` → `.app/Contents/Resources`），全部找不到时才提示失败；第三方
清单始终可用。

### `completions` — Shell 补全

生成补全脚本并写入 stdout：

```bash
ntfs-mac completions zsh  > ~/.zfunc/_ntfs-mac
ntfs-mac completions bash > /usr/local/etc/bash_completion.d/ntfs-mac
ntfs-mac completions fish > ~/.config/fish/completions/ntfs-mac.fish
```

## 配置文件

位置：`~/.config/ntfs-mac/config.toml`（遵循 XDG 规范）

```toml
# 默认挂载点基路径
mount_base = "/Volumes"

# ntfs-3g 额外挂载选项（空格分隔）
mount_options = ["noowners", "uid=501", "gid=20"]

# FUSE 驱动选择: "auto" | "macfuse" | "fuse-t"
fuse_driver = "auto"

# 格式化/修复前是否要求交互确认（脚本环境可设为 false）
require_confirmation = true

# 终端彩色输出
color = true
```

## GUI 构建

GUI 使用 Tauri v2 + 原生 HTML/CSS/JS（无 Node 框架依赖）：

```bash
# 开发模式（热重载）
cd crates/ntfs-mac-tauri/src-tauri
cargo tauri dev

# 构建 .app
cargo tauri build
# → bundles/macos/ntfs-mac.app
```

## macOS 安装包（.pkg）

一键构建包含 GUI + CLI 的完整安装包：

```bash
chmod +x scripts/build-macos.sh
./scripts/build-macos.sh
# → dist/ntfs-mac-0.1.0.pkg
```

产出物 `dist/ntfs-mac-<version>.pkg` 包含：

| 内容 | 安装位置 |
|------|----------|
| `ntfs-mac.app` (GUI) | `/Applications/ntfs-mac.app` |
| `ntfs-mac` (CLI) | `/usr/local/bin/ntfs-mac` |
| `LICENSE` (Apache-2.0 全文) | `/usr/local/share/ntfs-mac/LICENSE` |
| `NOTICE` (版权与第三方归属) | `/usr/local/share/ntfs-mac/NOTICE` |
| `THIRD_PARTY_LICENSES.md` | `/usr/local/share/ntfs-mac/THIRD_PARTY_LICENSES.md` |
| `README.md` | `/usr/local/share/ntfs-mac/README.md` |
| `install.sh` | `/usr/local/share/ntfs-mac/install.sh` |

> 载荷根目录即文件系统根目录，因此构建脚本会先把文件放进 `Applications/`、
> `usr/local/bin/`、`usr/local/share/ntfs-mac/` 再打包。构建结束时脚本会校验载荷
> 路径，若 `LICENSE` 之类的文件落到 `/` 会直接失败。

`/usr/local/share/ntfs-mac/` 正是 `ntfs-mac license --full` 查找许可证的目录之一，
所以安装后该命令可以直接读出完整文本。

安装器还带：
- **welcome.html** — 安装前介绍
- **readme.html** — 安装后使用指引
- **preinstall.sh** — macOS 版本检查（要求 ≥ 12.0）
- **postinstall.sh** — CLI 与许可证文件权限修正；升级时清理旧版残留在 `/` 的 app bundle

指定版本号：

```bash
VERSION=0.2.0 ./scripts/build-macos.sh
```

发布前签名与公证（绕过 Gatekeeper）：

```bash
codesign --deep --force --sign "Developer ID Installer: Your Name" dist/ntfs-mac-0.1.3.pkg
xcrun notarytool submit dist/ntfs-mac-0.1.3.pkg --wait
```

## 开发

```bash
# 快速检查（core + cli）
cargo check

# 运行测试
cargo test

# 格式化
cargo fmt --all

# Lint
cargo clippy --all

# 完整工作区（含 Tauri）
cargo check --workspace
cargo test --workspace
```

**测试策略**（60 个测试）：
- **41 个单元测试**（分布在 core 各模块内）：
  - `device.rs`：plist 解析（真实结构）+ mount 输出解析 + 卷信息增强
  - `daemon.rs`：守护进程 tick 逻辑 + LaunchAgent plist 生成
  - `config.rs`：round-trip 持久化测试
  - `deps.rs`：渲染路径测试
  - `format.rs`：label 验证测试
  - `mount.rs`：挂载选项构建 + unmount 错误处理
  - `fix.rs`：已挂载卷拒绝修复
  - `copy.rs`：dry-run 不 panic
  - `hardening.rs`：panic hook、信号标志、设备 ID 验证、挂载点验证、标签验证
- **1 个 plist fixture 测试**（`tests/device_fixture.rs`）：模拟 `diskutil list -plist` 输出，验证 NTFS 过滤和卷名提取
- **18 个集成测试**（`tests/integration.rs`）：runner 子进程（echo/false/stderr/timeout/缺失二进制）、which、config 持久化、输入校验矩阵、错误转换、退出码

## 生产硬化

### 崩溃恢复

panic hook 在安装时自动捕获崩溃信息（位置、消息、backtrace），写入 `~/Library/Logs/ntfs-mac.panics.log`。遇到 bug 时可直接将此文件提交报告。

### 信号处理

daemon 和 CLI 均通过 `signal-hook` 捕获 SIGINT / SIGTERM：
- daemon 每 500ms 轮询 `should_exit()` 标志，Ctrl+C 后 500ms 内优雅退出
- CLI 命令收到 SIGTERM 时返回 `Error::Cancelled`（退出码 3）

### 结构化日志

```bash
ntfs-mac --log-level debug doctor
```

日志写入 `~/Library/Logs/ntfs-mac.log`（含时间戳、模块、文件名、行号），stderr 输出简洁信息（无时间戳，适合人读）。通过 `NO_COLOR` 环境变量或 `--no-color` 可禁用彩色。

### 输入校验（fail-closed）

| 命令 | 校验内容 | 拒绝示例 |
|------|----------|----------|
| `mount`/`unmount`/`rw`/`format`/`fix` | 设备 ID 格式 `disk<N>` 或 `disk<N>s<M>` | `notadisk`、`disk 0`、`/dev/disk0` |
| `mount --mount-point` | 绝对路径 | `relative/path`、空串 |
| `format --label` | ≤32 字符，不含 `/` 或 `\` | `a/b`、空串 |

校验失败直接返回 `Error::InvalidArgument`（退出码 5），不会 panic。

### CI / CD

GitHub Actions 自动化工作流（`.github/workflows/ci.yml`）：

| Job | 运行环境 | 内容 |
|-----|----------|------|
| `lint-macos` | macOS 14 | `cargo fmt --check` + `cargo clippy` + `cargo test` |
| `build-macos` | macOS 14 | 安装 Tauri CLI + `build-macos.sh` + 上传 `.pkg` artifact |
| `audit` | Ubuntu 22.04 | `cargo audit`（已知 CVE 检查） |
| `licenses` | Ubuntu 22.04 | `cargo deny check licenses bans sources` + 校验 `THIRD_PARTY_LICENSES.md` 未过期 |

> `licenses` Job 依赖仓库根的 `deny.toml`。文件名不能改：cargo-deny 只识别
> `deny.toml` / `.deny.toml`，写成 `Cargo.deny.toml` 会被静默忽略并回退到默认策略
> （默认策略拒绝一切许可证，连 MIT 都会被判失败）。

### Homebrew Cask

cask 定义在 `Casks/ntfs-mac.rb`，安装 GitHub Releases 上的 `.pkg`
（GUI + CLI + 许可证文档一体安装），`brew style --cask` 零违规。

```bash
brew install --cask https://raw.githubusercontent.com/kodephp/rust_ntfs/main/Casks/ntfs-mac.rb
brew uninstall --cask ntfs-mac
```

发布新版本时需同步更新 cask 的 `version` 与 `sha256`
（`shasum -a 256 dist/ntfs-mac-<version>.pkg`）。

### 质量门

```bash
cargo fmt --all --check        # 格式化（零 diff）
cargo clippy --workspace        # Lint（零警告）
cargo test --workspace          # 测试（71/71 全绿）
cargo audit                     # 安全审计
cargo deny check                # 许可证 / bans / sources / advisories
./scripts/gen-third-party-licenses.sh --check   # 第三方清单未过期
```

## 安全设计

1. **破坏性操作强制确认**：`format` 要求 `DestructiveToken`，用户必须输入 `format <device> <label>` 才能继续。
2. **只读保护**：`mount -r` 可强制只读挂载，防止意外写入。
3. **命令隔离**：所有外部命令通过 `runner::run` 统一执行，带超时控制（SIGTERM → SIGKILL），无 shell 注入风险。
4. **配置白名单**：`mount_options` 选项白名单过滤，不接受任意字符串。
5. **无 unsafe 代码**：整个代码库零 `unsafe`，零 `unwrap` 在生产品路径上。

## 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般失败 |
| 2 | 缺少依赖 |
| 3 | 用户取消 |
| 4 | 确认不匹配 |
| 5 | 无效参数 |

## 常见问题

**Q: 挂载后 Mac 自动弹出？**
A: 系统可能尝试用原生只读驱动重新挂载。在终端执行 `mount -o nobrowse /Volumes/YourVolume` 可防止 Finder 重复弹出。

**Q: `ntfs-3g` 安装成功但 doctor 仍报缺失？**
A: 重新打开终端，或检查 `$PATH` 是否包含 `/usr/local/bin`（Intel）或 `/opt/homebrew/bin`（Apple Silicon）。

**Q: FUSE 内核扩展被系统阻止？**
A: 前往 系统设置 → 隐私与安全性 → 滚动到底部批准。Intel Mac 可能还需要在 系统偏好设置 → 安全与隐私 → 通用 中允许。

**Q: 格式化后 Windows 无法识别？**
A: 确保使用 `--quick` 快速格式化（默认）。`--sector-size` 与 `--cluster-size` 已在 0.1.2 起开放，取值受限（扇区 512/4096，簇 512–65536 的 2 的幂）；不确定时保持默认 4096。

## 许可证

Apache License 2.0 — 完整文本见 [LICENSE](LICENSE)。

本项目按 Apache-2.0 分发，因此还随附：

| 文件 | 作用 |
|------|------|
| [`LICENSE`](LICENSE) | Apache-2.0 全文 |
| [`NOTICE`](NOTICE) | 版权声明与第三方归属（Apache-2.0 §4(d) 要求再分发时保留） |
| [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) | 静态链接的 Rust 依赖及其许可证清单 |

命令行可随时查看：`ntfs-mac license`、`ntfs-mac license --full`、`ntfs-mac license --third-party`。

`THIRD_PARTY_LICENSES.md` 由脚本从 `Cargo.lock` 生成，请勿手工编辑：

```bash
scripts/gen-third-party-licenses.sh           # 重新生成
scripts/gen-third-party-licenses.sh --check   # 校验是否过期（CI 使用）
```

依赖许可证策略由 [`deny.toml`](deny.toml) 定义：所有许可证默认拒绝，仅 `allow`
列表中的逐项放行；MPL-2.0（`colored`、`option-ext` 等）通过具名 `exceptions` 单独
放行，而不是放宽全局白名单——这样升级依赖引入新的 MPL-2.0 crate 时 CI 会失败并
提示重新评估。

运行时通过子进程调用的 `ntfs-3g`、`macFUSE`/`FUSE-T`、`rsync` 等程序不随本项目
分发、也不与本项目链接，各自遵循其自身许可证；详见 [`NOTICE`](NOTICE)。
