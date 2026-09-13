# AGENTS.md — ntfs-mac 开发约束

本文档定义 ntfs-mac 项目的架构决策、设计规则和技术约束。
所有 AI 代理和开发者必须遵循这些规则。

## 项目定位

macOS NTFS 工具集——解决 macOS 原生只读 NTFS 的问题。
核心能力：挂载、卸载、格式化、修复、复制、自动挂载。
对标 Mounty，但更小、更可控、无闭源依赖。

## 架构

```
crates/
  ntfs-mac-core    核心库（subprocess 封装、plist 解析、配置、daemon、license 元数据）
  ntfs-mac-cli     CLI（clap 子命令）
  ntfs-mac-tauri   GUI（Tauri v2 + Vanilla HTML/JS）
```

- **core** 不依赖 tauri 或 clap，保持纯净
- **cli** 和 **tauri** 都依赖 core，不互相依赖
- workspace `default-members` 排除 tauri 以加速迭代

### 交付渠道与版本锁（勿改）

- **tauri 按 workspace 路径引用 core**（`ntfs-mac-core = { workspace = true }`
  → 根 `Cargo.toml` 的 `{ version, path = "crates/ntfs-mac-core" }`），
  **永不**从 crates.io 解析 core，也**永不**调用 CLI 二进制。
  因此 crates.io 上 `ntfs-mac-core@x` / `ntfs-mac-cli@x` 的版本对桌面应用**没有影响**。
- **tauri `publish = false`**：Tauri 应用不是 cargo-installable 产物，
  GUI 只通过 `.pkg` / Homebrew Cask 分发。crates.io 上永远只有 core 与 cli 两个包，
  这是预期状态，不要"补齐" tauri。
- **单一版本号**：根 `Cargo.toml` 只有一个 `version`，三个 crate 都用
  `version.workspace = true` 继承。升版只改根这一处，不可能出现桌面端与 CLI 错配。
  发版时仍需同步 `build-macos.sh` / `Casks/ntfs-mac.rb` / `README.md` 里写死的版本号。
- **`.pkg` 一体安装**：GUI + CLI + 许可证文档来自同一次构建，因此安装包内的
  `/usr/local/bin/ntfs-mac` 与 `/Applications/ntfs-mac.app` 必然同版本。

## 许可证与归属（Apache-2.0）

本项目以 Apache-2.0 分发，配套产物必须保持同步：

| 文件 | 位置 | 维护方式 |
|------|------|----------|
| `LICENSE` | 仓库根 | 手工，Apache-2.0 原文 |
| `NOTICE` | 仓库根 | 手工；版权 + 第三方归属。Apache-2.0 §4(d) 要求再分发时保留 |
| `THIRD_PARTY_LICENSES.md` | 仓库根 **与** `crates/ntfs-mac-core/` | **生成物**，由 `scripts/gen-third-party-licenses.sh` 从 `Cargo.lock` 生成 |
| `deny.toml` | 仓库根 | 手工；许可证策略（见下） |

要点：

- 所有 `.rs` 文件首行带 `// SPDX-License-Identifier: Apache-2.0` + 版权行。
- 两份 `THIRD_PARTY_LICENSES.md` 必须一致：根目录供人阅读，core 目录内那份由
  `include_str!` 编译进二进制，所以必须位于 crate 包内（否则 `cargo publish` 会
  因跨包路径而失败）。生成脚本一次写两份，避免漂移。
- 生成脚本对同名不同版本的 crate 按 `(name, version)` 排序，保证输出确定；
  `--check` 比较时会忽略 `- Generated:` 日期行，因此可以每天跑。
- `cargo publish` 的 `license` 字段来自 `Cargo.toml`；`license::SPDX_ID` 有单元
  测试与 `CARGO_PKG_LICENSE` 对齐。

### deny.toml 策略

- 所有许可证**默认拒绝**，仅 `allow` 内逐项放行（cargo-deny ≥ 0.18 移除了
  `allow-osi-fsf-free` / `deny` / `copyleft` / `default`）。
- MPL-2.0（`colored`、`option-ext`、`cssparser`、`selectors` 等）**不进全局白名单**，
  而是通过具名 `[[licenses.exceptions]]` 单独放行：MPL-2.0 是文件级弱 copyleft，
  可以随 Apache-2.0 作品分发，但引入新的 MPL crate 时应当重新评估，所以让检查失败。
- `[advisories] unmaintained = "workspace"`：只对**直接依赖**里的停维护 crate 报警。
  Tauri 传递依赖里的 `unic-*`、`proc-macro-error` 无升级路径，逐条 ignore 会变成
  六条需要人工维护的豁免；`cargo audit` 仍覆盖整图。
- **不要**把文件改名成 `Cargo.deny.toml`——cargo-deny 只认 `deny.toml` /
  `.deny.toml`，改名等于静默关闭所有许可证检查。

## 技术栈约束

| 维度 | 约束 |
|------|------|
| Rust | edition 2024, MSRV 1.85+ |
| 错误处理 | lib 用 thiserror（typed），app 层用 anyhow |
| 序列化 | serde + serde_json + toml |
| CLI | clap 4.x derive |
| GUI | Tauri v2 + Vanilla HTML/CSS/JS（无 Node 框架） |
| plist | plist crate（非 serde-xml-rs） |
| 依赖管理 | workspace.dependencies 集中，crate 用 `{ workspace = true }` |

## 设计规则（不可违反）

### 安全
1. **unsafe 白名单（仅 1 处）**：全代码库只允许 `runner::terminate_gracefully` 里的
   `unsafe { libc::kill(pid, SIGTERM) }`。原因：std 只有 `Child::kill()`（= SIGKILL），
   无法给持有文件系统状态的工具（`fsck_ntfs` / `newfs_ntfs` / `rsync` / `cp`）
   先发 SIGTERM 的机会。新增任何 `unsafe` 必须写明 `SAFETY` 契约，并同步更新
   本条与 `crates/ntfs-mac-core/src/lib.rs` 顶部的规则清单
2. **零生产 unwrap**：生产路径不允许 `unwrap()`，测试中可以；锁获取用 `unwrap_or_else(PoisonError::into_inner)` 容忍 poison
3. **破坏性操作强制确认**：format/erase 需要 `DestructiveToken` + 用户输入确认短语
4. **命令隔离**：所有外部命令经 `runner::run` 统一执行，带 timeout，无 shell 注入；`RunOptions.capture` 默认 `true`（手动 Default），`false` 才继承 stdio
5. **mount 选项校验**：`mount::validate_mount_options` 拒绝含空白/分隔符/引号等注入面字符的选项，config set 与挂载时双重把关

### 代码风格
6. **PSR12 等价**：rustfmt 默认规则，不手写格式化
7. **命名简短**：函数/变量命名精炼，模块名简短
8. **注释用英文**：代码注释和文档字符串用英文
9. **测试覆盖**：纯函数必须有单元测试，subprocess 调用只测解析层

### 依赖
10. **不引入新依赖前必须说明理由**：为什么 std 或现有依赖不够
11. **feature 最小化**：只启用需要的 feature
12. **无 git 依赖**：所有依赖必须是 crates.io 发布版本

### 构建
13. **CI 矩阵**：aarch64-apple-darwin + x86_64-apple-darwin
14. **release profile**：lto = "thin", strip = "debuginfo"
15. **Cargo.lock 提交**：binary 项目提交锁文件

## 禁止事项

- ❌ 不引入 async runtime（tokio 等）
- ❌ 不使用 FFI 绑定（不写 ntfs-3g 的 FFI）
- ❌ 不引入 Node.js 框架（GUI 只用 Vanilla）
- ❌ 不修改 vendor 包
- ❌ 不在 core 中依赖 clap 或 tauri
- ❌ 不使用 `--all-features` 跑测试
- ❌ 不自动 git commit/push（需用户明确指令）
- ❌ 不提交 `.workbuddy/`、`target/`、`node_modules/`

## 命令速查

```bash
# 快速检查（core + cli）
cargo check

# 测试
cargo test

# 格式化
cargo fmt

# Lint
cargo clippy

# 完整工作区
cargo check --workspace

# Release 构建
cargo build --release -p ntfs-mac-cli
cargo build --release -p ntfs-mac-tauri

# GUI 开发模式
cd crates/ntfs-mac-tauri/src-tauri && cargo tauri dev

# GUI 构建 .app
cd crates/ntfs-mac-tauri/src-tauri && cargo tauri build

# 前端质量门（JS 语法 + 中英文契约 + DOM id 一致性）
scripts/test-frontend.sh

# 生成应用图标与菜单栏模板图
scripts/gen-icons.py

# 许可证 / 第三方归属
cargo deny check                       # deny.toml 策略（licenses/bans/sources/advisories）
./scripts/gen-third-party-licenses.sh           # 重新生成 THIRD_PARTY_LICENSES.md
./scripts/gen-third-party-licenses.sh --check   # 校验是否过期
cargo run -p ntfs-mac-cli -- license            # 人工查看许可证摘要
```

## 发布流程

1. 实现 → `cargo test --workspace` 全绿
2. `cargo clippy --workspace --all-targets -- -D warnings` 无 error
3. `cargo fmt --all -- --check` 零差异
4. `scripts/test-frontend.sh` 通过（前端中英文契约）
5. `cargo deny check` 通过
6. 更新版本号（Cargo.toml workspace.package.version，同步 build-macos.sh / Casks / README）
7. `git add <指定文件>`（禁用 `git add -A`）
8. `git commit` + `git tag` + `git push`（需用户确认）

## 退出码规范

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般失败 |
| 2 | 缺少依赖 |
| 3 | 用户取消 |
| 4 | 确认不匹配 |
| 5 | 无效参数 |

## 配置文件

路径：`~/.config/ntfs-mac/config.toml`（XDG 规范）
字段：mount_base, mount_options, fuse_driver, require_confirmation, color
手动实现 `Default`（不用 derive），确保默认值明确。

## 外部命令

| 命令 | 用途 |
|------|------|
| diskutil | 挂载/卸载/设备信息 |
| hdiutil | 磁盘镜像 |
| ntfs-3g | NTFS 读写挂载 |
| newfs_ntfs / mkntfs | 格式化 NTFS |
| ntfsfix | 快速修复 |
| fsck_ntfs | 彻底修复 |
| rsync | 文件复制（带进度） |
| sw_vers | 系统版本检测 |

所有命令通过 `runner::run` 执行，timeout 默认 30s（格式化 60s）。

## GUI 前端约束（`crates/ntfs-mac-tauri/src`）

### 入口方式（勿改）

- **必须**用 `window.__TAURI_INTERNALS__.invoke` / `internals.event.listen` 调用后端。
  Tauri v2 默认**不注入** `window.__TAURI__`，写 `__TAURI__.core` 会导致窗口空白。
- `invoke` 参数用 camelCase（`deviceId` → `device_id`、`useFsck` → `use_fsck`）。

### 菜单栏托盘（`lib.rs`）

- 托盘 id `ntfs-mac`，菜单 id `ntfs-mac-menu`，5s 轮询重建，签名去重避免重建 `NSMenu`。
- 扁平 mounty 式布局：卷列表 → 依赖状态 → 刷新 / 主窗口 / 安装依赖 → 退出。
- 每卷菜单项 id 带前缀：`vol-mount-` / `vol-open-` / `vol-unmount-` / `vol-eject-` + 设备号。
- 关闭窗口**不得**退出进程（`CloseRequested` → `prevent_close()` + `hide()`）；
  退出只走托盘 `quit`。
- 托盘图标必须是纯黑 + alpha 模板图（`icon_as_template(true)`），由 `scripts/gen-icons.py` 生成。

### 中英文 i18n（默认中文）

- `lib.rs` 的 `Labels` 结构体与 `app.js` 的 `I18N` 字典必须**逐键逐值一致**
  （`header` / `empty` / `mount` / `open` / `unmount` / `eject` / `refresh` /
  `show_window` / `install_deps` / `quit` / `ready` / `missing` / `volumes_n` / `mounted_n`）。
- `app.js` 的 `I18N[lang].ui` 只放窗口专用文案，不进 `Labels`。
- `scripts/test-frontend.sh` 会对比两边，漂移即失败。改文案必须同步改两处。
- 语言存 `localStorage["ntfs-mac.language"]`，后端存 `AppState.language`；
  `labels()` / `set_language` 对非 `"en"` 一律回落到中文。
- `index.html` 必须 `lang="zh"`，静态可见文案全部为中文。

### 界面 ↔ API 安全性（v0.1.6 起强制）

- **禁止无界子进程**：任何 `runner::run` 必须带 `timeout: Some(...)`。
  `RunOptions::default().timeout == None` 会走无界 `child.wait()`——UI 上一个
  卡住的命令就等于永久转圈。只读系统探针用 `deps.rs::PROBE_TIMEOUT`（5s），
  主窗口操作用 `CMD_TIMEOUT`（30s）。新增调用必须给上限。
- **前端每个 `invoke` 必须走 `call()`**：`app.js` 里不允许出现裸 `invoke("...")`，
  必须在 `CALL_TIMEOUT_MS` 里登记该命令的毫秒上限（取值需高于后端自身的子进程
  上限，避免误伤慢盘；但必须有限）。
- **响应必须做形状校验**：用 `expectShape()` + `validDepReport()` /
  `validVolumeList()` 校验后端返回结构；漂移时显示「接口返回异常」而非渲染
  一片 `undefined`。新增命令返回新结构时同步加校验。
- **失败必须可恢复**：依赖与卷两个 loader 走 `showError()` 渲染错误 +
  「重新检测」按钮，禁止静默吞掉错误。两个 loader 各自用
  `depsInFlight` / `volumesInFlight` 合并 5s `ntfs-mac:refresh` 的重复触发
  ——不加合并就会堆叠并发探针，表现为"永远检测中"。
- **单实例**：`run()` 必须挂 `tauri_plugin_single_instance`，回调里调
  `show_window()` 聚焦既有窗口。关窗只隐藏不退出，因此旧版本残留进程会被托盘
  留住；没有单实例守护就会开出第二个窗口。窗口 label 固定为 `"main"`。
- `scripts/test-frontend.sh` 共 7 项契约检查，含「无裸 invoke」「每命令有超时
  上限」「UI 调用的命令集与 `generate_handler!` 注册的完全一致」。新增命令时
  三处（后端 handler、前端 `call()`、`CALL_TIMEOUT_MS`）必须同时更新。

### 子进程超时契约（v0.1.7 起强制，覆盖全仓库）

- **超时上限是硬保证**：`runner::run` 的 `timeout` 一旦到期就必定返回，不会
  被子进程拖住。两层守护：SIGTERM 后给 `SIGTERM_GRACE`（1s）优雅退出，
  stdout/stderr 读取线程的 join 上限为 `READ_DRAIN_TIMEOUT`（500ms）。
  没有 drain 上限时，超时的 `brew install` 会留下继承管道写端的 `curl`/`tar`，
  让 `run()` 卡到孤进程退出为止（实测曾卡满 60s）——测试
  `runner_timeout_survives_orphaned_grandchildren` 就是防这个回退。
- **先 SIGTERM 再 SIGKILL**：`terminate_gracefully` 先发 SIGTERM。这些工具
  （`fsck_ntfs` 检查、`newfs_ntfs` 格式化、`rsync`/`cp` 传输最长 1h）运行时
  持有文件系统状态，直接 SIGKILL 比干净退出留下更糟的卷状态。行为由
  `runner_timeout_sends_sigterm_before_sigkill` 用 shell trap 写 marker 文件验证。
- **超时不泄漏内部哨兵**：`runner::run` 在超时时返回 `timed_out = true` +
  `status = TIMEOUT_STATUS`（150，已导出常量，不写魔法数字）；
  `run_expect_success` 把它转成 `Error::CommandTimedOut { cmd, timeout }`。
  用户看到的是「command `fsck_ntfs` timed out after 30s」，
  而不是「exited with status 150」+ 空 stderr。CLI 退出码走既有兜底（1）。
- **新增子进程调用必须带 timeout**：`RunOptions::default().timeout == None`
  即无界 `child.wait()`。全仓库 `src/` 下不允许出现 `RunOptions::default()`
  作为实参（只有测试里可以用，测试命令都是 `echo`/`false`/`sleep`）。

### 禁止

- 前端不得新增构建步骤（无 npm / bundler），保持纯 HTML/CSS/JS。
- 不得把赞助/收款二维码写回界面（已随 v0.1.5 移除，含 `reveal_sponsor_qr` 命令与 CLI `sponsor` 子命令）。

## 安装包构建

### 脚本

`scripts/build-macos.sh` 一键构建完整 `.pkg`：

```bash
./scripts/build-macos.sh                # 默认 v0.1.0
VERSION=0.2.0 ./scripts/build-macos.sh  # 指定版本
```

产出：`dist/ntfs-mac-<version>.pkg`（~3.6M）

### pkgbuild / productbuild 关键陷阱

构建过程使用 `pkgbuild --root` 生成组件包，再用 `productbuild --distribution` 组装。

**陷阱 1：pkgbuild 输出路径必须位于 `--package-path` 内**

```bash
# ✓ 正确：组件包在 --package-path 指向的目录内
pkgbuild --root dist/pkg-payload --identifier com.kodephp.ntfs-mac --version 0.1.0 \
         dist/packages/ntfs-mac-component-0.1.0.pkg
productbuild --distribution dist/distribution.xml --package-path dist/packages ...

# ✗ 错误：组件包在 dist/ 但 --package-path 指向 dist/packages → productbuild 静默产出 4K 空 pkg
pkgbuild --root dist/pkg-payload ... dist/ntfs-mac-component-0.1.0.pkg
productbuild --distribution dist/distribution.xml --package-path dist/packages ...
```

**陷阱 2：distribution.xml 必须包含完整 choices-outline 结构**

productbuild 静默丢弃没有 `<choices-outline>` + `<choice>` + `<pkg-ref>` 三段结构的 distribution。仅写 `<pkg-ref id="..." version="...">file.pkg</pkg-ref>` 会导致 4K 空 pkg（但 productbuild 返回成功码，无警告）。

最小可用的 distribution.xml 结构：

```xml
<installer-gui-script minSpecVersion="2">
    <title>...</title>
    <welcome file="welcome.html" mime-type="text/html"/>
    <readme file="readme.html" mime-type="text/html"/>
    <scripts>
        <preinstall file="preinstall.sh" mime-type="text/x-sh"/>
        <postinstall file="postinstall.sh" mime-type="text/x-sh"/>
    </scripts>
    <options customize="allow" require-scripts="false" rootVolumeOnly="false"/>
    <choices-outline>
        <line choice="default">
            <line choice="com.kodephp.ntfs-mac"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="com.kodephp.ntfs-mac" visible="false">
        <pkg-ref id="com.kodephp.ntfs-mac" onConclusion="none"/>
    </choice>
    <pkg-ref id="com.kodephp.ntfs-mac" version="0.1.0" onConclusion="none">ntfs-mac-component-0.1.0.pkg</pkg-ref>
</installer-gui-script>
```

替代方案：`productbuild --synthesize --package <component.pkg> <out.xml>` 自动生成基础 XML，再手动注入 `<welcome>` `<readme>` `<scripts>` `<title>`。

**陷阱 3：pkgbuild 不会把 `.app` 自动搬到 `/Applications`**

曾经这里写的是"payload 中含 `.app` 时 pkgbuild 会自动把 `.app` 安装到 `/Applications`"。
**这是错的**，并直接导致了 0.1.0–0.1.2 的载荷把 `ntfs-mac.app`、`LICENSE`、`README.md`
全部装进文件系统根目录。实测（`pkgutil --payload-files dist/ntfs-mac-0.1.2.pkg`）：

```
./LICENSE
./README.md
./ntfs-mac-cli
./ntfs-mac.app/Contents/...
./scripts/install.sh
```

`pkgbuild --root <dir>` 的行为是：**把 `<dir>` 当作文件系统根目录**，内部结构原样保留，
`--install-location` 默认 `/`。所以要装到哪里，就必须先在 payload 里搭出对应目录：

```
dist/pkg-payload/
  Applications/ntfs-mac.app
  usr/local/bin/ntfs-mac
  usr/local/share/ntfs-mac/{LICENSE,NOTICE,THIRD_PARTY_LICENSES.md,README.md,install.sh}
```

`build-macos.sh` 在打包后会校验 `pkgutil --payload-files`：既检查必需路径存在，
也检查 `./LICENSE`、`./ntfs-mac.app` 之类的路径**不**存在，防止回归。

### 验证

```bash
pkgutil --payload-files dist/ntfs-mac-0.1.5.pkg    # 只应出现 Applications/ 与 usr/local/
xar -tf dist/ntfs-mac-0.1.5.pkg | head             # 查看顶层结构
pkgutil --check-signature dist/ntfs-mac-0.1.5.pkg  # 签名检查（未签名会退出码 1）
```

## 发布检查清单（Release Checklist）

每次发布前按顺序执行：

### 1. 代码质量门

```bash
cargo fmt --all --check                        # 格式化
cargo clippy --workspace --all-targets         # Lint（必须 0 警告）
cargo test --workspace                         # 全部测试通过（当前 71）
cargo audit                                    # 安全检查
cargo deny check                               # 许可证 / bans / sources / advisories
./scripts/gen-third-party-licenses.sh --check  # 第三方清单未过期
```

> `cargo deny` 读取仓库根的 **`deny.toml`**。文件名不能写成 `Cargo.deny.toml`：
> cargo-deny 不认这个名字，会打印 `unable to find a config path, falling back to
> default config` 并用默认策略（拒绝一切许可证）判失败。
>
> cargo-deny ≥ 0.18 已移除 `allow-osi-fsf-free` / `deny` / `copyleft` / `default`，
> 所有许可证必须在 `allow` 中逐项列出。

### 2. 构建

```bash
./scripts/build-macos.sh             # 产出 dist/ntfs-mac-<version>.pkg
```

> 载荷根目录即文件系统根目录，`build-macos.sh` 会把文件放进 `Applications/`、
> `usr/local/bin/`、`usr/local/share/ntfs-mac/`。脚本末尾会校验载荷路径，任何
> 落到 `/` 的文件都会让构建失败。

### 3. 验证安装包

```bash
pkgutil --payload-files dist/ntfs-mac-*.pkg    # 必须只出现 Applications/ 与 usr/local/
pkgutil --check-signature dist/ntfs-mac-*.pkg   # 签名检查（未签名返回码 1）
```

### 4. 版本号

版本号需同步以下位置（`Cargo.lock` 由 cargo 自动更新）：

```bash
# Cargo.toml        → [workspace.package] version
# Cargo.toml        → [workspace.dependencies] ntfs-mac-core 的 version
# scripts/build-macos.sh → VERSION 默认值
# Casks/ntfs-mac.rb      → version 与 sha256（.pkg 生成后回填）
```

发布说明写在 GitHub Release 正文中，仓库内不再维护单独的 CHANGELOG。

### 5. 签名 + 公证（发布用）

```bash
# 需要 Apple Developer ID
codesign --deep --force --sign "Developer ID Installer: <Your Name>" dist/ntfs-mac-*.pkg
xcrun notarytool submit dist/ntfs-mac-*.pkg --wait --keychain-profile <profile>
```

### 6. 提交 + 打 tag

```bash
git add -A
git commit -m "release: v<version>"
git tag -a v<version> -m "Release v<version>"
git push origin main --tags
```

### 7. GitHub Release

在 GitHub 上创建 Release，附上 `dist/ntfs-mac-<version>.pkg`。

### 8. Homebrew Cask

```bash
# 回填 sha256 后校验（cask 需在 tap 内才能被 brew 识别，本地验证可用临时 tap）
shasum -a 256 dist/ntfs-mac-<version>.pkg
TAP="$(brew --repository)/Library/Taps/kodephp/homebrew-style-tmp"
mkdir -p "$TAP/Casks" && cp Casks/ntfs-mac.rb "$TAP/Casks/"
git -C "$TAP" init -q . && git -C "$TAP" add Casks/ntfs-mac.rb
brew style --cask kodephp/style-tmp/ntfs-mac
brew info   --cask kodephp/style-tmp/ntfs-mac   # 解析校验
rm -rf "$TAP" && rmdir "$(brew --repository)/Library/Taps/kodephp"
```

cask 通过 `pkg` stanza 安装 Releases 上的 `.pkg`，
`uninstall`/`zap` 以 `pkgutil: "com.kodephp.ntfs-mac"` 精确注销。
