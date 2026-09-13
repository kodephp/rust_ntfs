/* SPDX-License-Identifier: Apache-2.0
 * Copyright 2026 kodephp contributors
 *
 * ntfs-mac GUI frontend. Plain HTML/CSS/JS, no build step, no framework.
 *
 * Tauri v2 does not inject `window.__TAURI__` by default, so the only stable
 * entry point is `window.__TAURI_INTERNALS__` — `invoke()` for commands and
 * `event.listen()` for backend-pushed updates.
 */

const internals = window.__TAURI_INTERNALS__;
const invoke = internals.invoke;
const listen = internals.event.listen;

// ---------------------------------------------------------------------------
// i18n — Chinese by default.
//
// The keys outside `ui` mirror `Labels` in `src-tauri/src/lib.rs` exactly; the
// two are compared in `scripts/test-frontend.sh` so the menu-bar and the window
// can never drift apart. Keep them in sync.
// ---------------------------------------------------------------------------

const I18N = {
    zh: {
        header: "NTFS 卷",
        empty: "未检测到 NTFS 卷",
        mount: "挂载",
        open: "在 Finder 中打开",
        unmount: "卸载",
        eject: "弹出移动硬盘",
        refresh: "刷新",
        show_window: "打开主窗口",
        install_deps: "安装依赖",
        quit: "退出 ntfs-mac",
        ready: "依赖已就绪",
        missing: "依赖缺失",
        volumes_n: "{n} 个 NTFS 卷",
        mounted_n: "已挂载 {n} 个",
        ui: {
            deps_title: "系统状态",
            volumes_title: "NTFS 卷",
            foot_hint: "关闭窗口不会退出；退出请点菜单栏图标。",
            loading: "检测中…",
            deps_ready: "所有依赖就绪",
            deps_partial: "部分依赖缺失",
            dep_ok: "可用",
            dep_no: "缺失",
            no_deps: "未能读取依赖状态",
            vol_empty: "未检测到 NTFS 卷。插入移动硬盘后会自动刷新。",
            vol_loading: "正在扫描磁盘…",
            vol_error: "扫描失败：{msg}",
            vol_count: "{n} 个卷 · 已挂载 {m} 个",
            vol_mounted: "已挂载",
            vol_unmounted: "未挂载",
            vol_external: "外接",
            vol_internal: "内置",
            vol_mounted_at: "挂载点：{path}",
            vol_size: "容量：{size}",
            vol_type: "类型：{type}",
            vol_dev: "设备：{id}",
            vol_location: "位置：{loc}",
            mount_ro: "只读挂载",
            fix: "修复",
            fix_fsck: "深度修复",
            format: "格式化",
            fix_note: "修复磁盘错误（需先卸载）",
            format_warn: "⚠️ 此操作将清空该卷上的全部数据，且无法撤销。",
            format_label: "卷标（最多 11 个字符）",
            format_confirm: "格式化",
            format_doing: "正在格式化…",
            action_pending: "正在执行…",
            msg_mounted_at: "已挂载到 {path}",
            msg_mounted_ro: "已只读挂载到 {path}",
            msg_unmounted: "已卸载 {id}",
            msg_ejected: "已弹出 {path}",
            msg_fixed: "修复完成：{id}",
            msg_opened: "已在 Finder 中打开 {path}",
            msg_format_ok: "格式化完成",
            msg_err_mount: "挂载失败：{msg}",
            msg_err_unmount: "卸载失败：{msg}",
            msg_err_eject: "弹出失败：{msg}",
            msg_err_fix: "修复失败：{msg}",
            msg_err_open: "打开失败：{msg}",
            msg_err_format: "格式化失败：{msg}",
            msg_err_deps: "检测依赖失败：{msg}",
            msg_err_list: "扫描卷失败：{msg}",
            msg_err_lang: "切换语言失败：{msg}",
            ok: "成功",
            about_title: "关于",
            about_desc: "说明",
            about_repo: "仓库",
            about_license: "许可证",
            about_authors: "作者",
            about_version: "版本",
            modal_cancel: "取消",
            modal_confirm: "确认",
        },
    },
    en: {
        header: "NTFS volumes",
        empty: "No NTFS volumes found",
        mount: "Mount",
        open: "Open in Finder",
        unmount: "Unmount",
        eject: "Eject drive",
        refresh: "Refresh",
        show_window: "Show main window",
        install_deps: "Install dependencies",
        quit: "Quit ntfs-mac",
        ready: "All dependencies ready",
        missing: "Dependencies missing",
        volumes_n: "{n} volume(s)",
        mounted_n: "{n} mounted",
        ui: {
            deps_title: "System status",
            volumes_title: "NTFS volumes",
            foot_hint: "Closing the window does not quit; use the menu-bar icon.",
            loading: "Checking…",
            deps_ready: "All dependencies ready",
            deps_partial: "Some dependencies missing",
            dep_ok: "available",
            dep_no: "missing",
            no_deps: "Could not read dependency status",
            vol_empty: "No NTFS volumes found. Plug in a drive and it will refresh automatically.",
            vol_loading: "Scanning disks…",
            vol_error: "Scan failed: {msg}",
            vol_count: "{n} volume(s) · {m} mounted",
            vol_mounted: "mounted",
            vol_unmounted: "unmounted",
            vol_external: "external",
            vol_internal: "internal",
            vol_mounted_at: "Mounted at {path}",
            vol_size: "Size: {size}",
            vol_type: "Type: {type}",
            vol_dev: "Device: {id}",
            vol_location: "Location: {loc}",
            mount_ro: "Mount read-only",
            fix: "Fix",
            fix_fsck: "Deep fix",
            format: "Format",
            fix_note: "Fix disk errors (must be unmounted)",
            format_warn: "⚠️ This will ERASE all data on the volume and cannot be undone.",
            format_label: "Volume label (max 11 chars)",
            format_confirm: "Format",
            format_doing: "Formatting…",
            action_pending: "Working…",
            msg_mounted_at: "Mounted at {path}",
            msg_mounted_ro: "Mounted read-only at {path}",
            msg_unmounted: "Unmounted {id}",
            msg_ejected: "Ejected {path}",
            msg_fixed: "Fixed: {id}",
            msg_opened: "Opened in Finder: {path}",
            msg_format_ok: "Volume formatted",
            msg_err_mount: "Mount failed: {msg}",
            msg_err_unmount: "Unmount failed: {msg}",
            msg_err_eject: "Eject failed: {msg}",
            msg_err_fix: "Fix failed: {msg}",
            msg_err_open: "Open failed: {msg}",
            msg_err_format: "Format failed: {msg}",
            msg_err_deps: "Dependency check failed: {msg}",
            msg_err_list: "Volume scan failed: {msg}",
            msg_err_lang: "Language switch failed: {msg}",
            ok: "Done",
            about_title: "About",
            about_desc: "Description",
            about_repo: "Repository",
            about_license: "License",
            about_authors: "Authors",
            about_version: "Version",
            modal_cancel: "Cancel",
            modal_confirm: "Confirm",
        },
    },
};

const LANG_KEY = "ntfs-mac.language";

// ---------------------------------------------------------------------------
// i18n runtime
// ---------------------------------------------------------------------------

let lang = localStorage.getItem(LANG_KEY) || "zh";
if (!Object.prototype.hasOwnProperty.call(I18N, lang)) {
    lang = "zh";
}

/** Resolve a dotted key (`ui.deps_title`) against the current language. */
function tr(key, vars) {
    let node = I18N[lang] || I18N.zh;
    for (const part of key.split(".")) {
        node = node[part];
        if (node === undefined) {
            return key;
        }
    }
    if (typeof node !== "string") {
        return key;
    }
    if (vars === undefined) {
        return node;
    }
    return node.replace(/\{(\w+)\}/g, (whole, name) =>
        vars[name] === undefined ? whole : String(vars[name]));
}

/** Escape a value before it is placed into `innerHTML`. */
function esc(value) {
    return String(value ?? "")
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

/**
 * Re-translate every element carrying `data-i18n`. Called on language change
 * and after each dynamic render.
 */
function applyI18n() {
    document.documentElement.lang = lang === "en" ? "en" : "zh";
    for (const el of document.querySelectorAll("[data-i18n]")) {
        el.textContent = tr(el.dataset.i18n);
    }
    document.getElementById("modal-cancel").textContent = tr("ui.modal_cancel");
    for (const btn of ["lang-zh", "lang-en"]) {
        document.getElementById(btn).classList.toggle("lang-active",
            btn === (lang === "en" ? "lang-en" : "lang-zh"));
    }
}

async function setLang(next) {
    if (next === lang) {
        return;
    }
    try {
        await invoke("set_language", { language: next });
    } catch (err) {
        toast(tr("ui.msg_err_lang", { msg: err }), "error");
        return;
    }
    lang = next;
    localStorage.setItem(LANG_KEY, next);
    applyI18n();
    await loadDeps();
    await loadVolumes();
}

// ---------------------------------------------------------------------------
// Toasts
// ---------------------------------------------------------------------------

function toast(message, kind = "info", ms = 4200) {
    const container = document.getElementById("toast-container");
    const node = document.createElement("div");
    node.className = `toast toast-${kind}`;
    node.textContent = message;
    container.appendChild(node);
    setTimeout(() => node.remove(), ms);
}

function flashError(message) {
    toast(message, "error", 6000);
}

// ---------------------------------------------------------------------------
// Dependencies
// ---------------------------------------------------------------------------

let depsReady = true;

async function loadDeps() {
    const list = document.getElementById("deps-list");
    const badge = document.getElementById("deps-badge");
    const installBtn = document.getElementById("install-btn");
    list.innerHTML = `<div class="empty-hint">${esc(tr("ui.loading"))}</div>`;

    let report;
    try {
        report = await invoke("check_deps");
    } catch (err) {
        list.innerHTML = `<div class="row-error">${esc(tr("ui.msg_err_deps", { msg: err }))}</div>`;
        badge.textContent = tr("ui.deps_partial");
        badge.className = "badge badge-warn";
        installBtn.hidden = false;
        depsReady = false;
        return;
    }

    const macos = report.macos_version ? `macOS ${esc(report.macos_version)}` : "macOS";
    document.getElementById("platform-info").innerHTML =
        `${macos} · <span class="mono">${esc(report.arch)}</span>`;

    depsReady = Boolean(report.ready);
    const missing = (report.deps || []).filter((d) => !d.present);
    badge.textContent = depsReady ? tr("ready") : tr("missing");
    badge.className = depsReady ? "badge badge-ok" : "badge badge-warn";
    installBtn.hidden = depsReady;

    const rows = (report.deps || []).map((dep) => {
        const mark = dep.present ? "✓" : "✕";
        const cls = dep.present ? "dep-ok" : "dep-no";
        const state = dep.present ? tr("ui.dep_ok") : tr("ui.dep_no");
        const path = dep.path ? `<span class="path mono">${esc(dep.path)}</span>` : "";
        const hint = !dep.present && dep.install_hint
            ? `<div class="dep-hint">${esc(dep.install_hint)}</div>` : "";
        return `<div class="dep-row">
            <span class="dep-mark ${cls}">${mark}</span>
            <span class="dep-name">${esc(dep.name)}</span>
            ${path}
            <span class="dep-state ${cls}">${state}</span>
            ${hint}
        </div>`;
    });
    list.innerHTML = rows.length
        ? rows.join("")
        : `<div class="empty-hint">${esc(tr("ui.no_deps"))}</div>`;
}

// ---------------------------------------------------------------------------
// Volumes
// ---------------------------------------------------------------------------

let volumes = [];

async function loadVolumes() {
    const el = document.getElementById("volumes-list");
    const count = document.getElementById("volumes-count");

    let next;
    try {
        next = await invoke("list_volumes");
    } catch (err) {
        el.innerHTML = `<div class="row-error">${esc(tr("ui.vol_error", { msg: err }))}</div>`;
        count.textContent = "–";
        volumes = [];
        return;
    }

    volumes = Array.isArray(next) ? next : [];
    const mounted = volumes.filter((v) => v.mounted).length;
    count.textContent = volumes.length
        ? tr("ui.vol_count", { n: volumes.length, m: mounted })
        : tr("empty");

    if (volumes.length === 0) {
        el.innerHTML = `<div class="empty-hint">${esc(tr("ui.vol_empty"))}</div>`;
        return;
    }
    el.innerHTML = volumes.map(volumeCard).join("");
}

/** One volume card. Every action is guarded behind the backend validator. */
function volumeCard(vol) {
    const id = esc(vol.device_identifier);
    const mounted = Boolean(vol.mounted);
    const location = vol.location === "internal" ? tr("ui.vol_internal") : tr("ui.vol_external");
    const kind = mounted ? tr("ui.vol_mounted") : tr("ui.vol_unmounted");

    const meta = [
        `<span class="meta">${esc(tr("ui.vol_size", { size: vol.size_pretty }))}</span>`,
        `<span class="meta">${esc(tr("ui.vol_type", { type: vol.media_type }))}</span>`,
        `<span class="meta">${esc(tr("ui.vol_dev", { id: vol.device_identifier }))}</span>`,
        `<span class="meta">${esc(tr("ui.vol_location", { loc: location }))}</span>`,
    ].join("");

    const mountInfo = vol.mount_point
        ? `<div class="mount-info">${esc(tr("ui.vol_mounted_at", { path: vol.mount_point }))}</div>`
        : "";

    const buttons = [
        `<button class="btn btn-small" type="button" data-act="open" data-dev="${id}">
            ${esc(tr("open"))}</button>`,
        mounted
            ? `<button class="btn btn-small btn-warn" type="button" data-act="unmount" data-dev="${id}">
                ${esc(tr("unmount"))}</button>`
            : `<button class="btn btn-small btn-ok" type="button" data-act="mount" data-dev="${id}">
                    ${esc(tr("mount"))}</button>
               <button class="btn btn-small" type="button" data-act="mount-ro" data-dev="${id}">
                    ${esc(tr("ui.mount_ro"))}</button>`,
        mounted ? "" : `<button class="btn btn-small" type="button" data-act="fix" data-dev="${id}">
                ${esc(tr("ui.fix"))}</button>`,
        mounted ? "" : `<button class="btn btn-small" type="button" data-act="fix-fsck" data-dev="${id}">
                ${esc(tr("ui.fix_fsck"))}</button>`,
        `<button class="btn btn-small btn-danger" type="button" data-act="eject" data-dev="${id}">
            ${esc(tr("eject"))}</button>`,
        mounted ? "" : `<button class="btn btn-small btn-danger" type="button" data-act="format" data-dev="${id}">
            ${esc(tr("ui.format"))}</button>`,
    ].join("");

    return `<div class="volume-card">
        <div class="volume-icon">${mounted ? "💾" : "📀"}</div>
        <div class="volume-main">
            <div class="volume-title">
                <span class="volume-name">${esc(vol.display_label)}</span>
                <span class="badge ${mounted ? "badge-ok" : "badge-muted"}">${esc(kind)}</span>
            </div>
            <div class="volume-meta">${meta}</div>
            ${mountInfo}
            <div class="volume-actions">${buttons}</div>
        </div>
    </div>`;
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/** Disable all buttons for the duration of one command. */
function busy(on) {
    for (const btn of document.querySelectorAll("button")) {
        btn.disabled = on;
    }
}

function errText(kind, msg) {
    const map = {
        mount: "ui.msg_err_mount",
        unmount: "ui.msg_err_unmount",
        eject: "ui.msg_err_eject",
        fix: "ui.msg_err_fix",
        open: "ui.msg_err_open",
        format: "ui.msg_err_format",
    };
    return tr(map[kind] || "ui.msg_err_mount", { msg });
}

async function runAction(act, deviceId, button) {
    const original = button ? button.textContent : "";
    if (button) {
        button.disabled = true;
        button.textContent = tr("ui.action_pending");
    }
    try {
        switch (act) {
            case "mount": {
                const at = await invoke("mount_volume", { deviceId, readonly: false });
                toast(tr("ui.msg_mounted_at", { path: at }), "ok");
                break;
            }
            case "mount-ro": {
                const at = await invoke("mount_volume", { deviceId, readonly: true });
                toast(tr("ui.msg_mounted_ro", { path: at }), "ok");
                break;
            }
            case "unmount":
                await invoke("unmount_volume", { deviceId });
                toast(tr("ui.msg_unmounted", { id: deviceId }), "ok");
                break;
            case "open": {
                const at = await invoke("open_in_finder", { deviceId });
                toast(tr("ui.msg_opened", { path: at }), "ok");
                break;
            }
            case "fix":
                await invoke("fix_volume", { deviceId, useFsck: false });
                toast(tr("ui.msg_fixed", { id: deviceId }), "ok");
                break;
            case "fix-fsck":
                await invoke("fix_volume", { deviceId, useFsck: true });
                toast(tr("ui.msg_fixed", { id: deviceId }), "ok");
                break;
            case "eject": {
                const at = await invoke("eject_volume", { deviceId });
                toast(tr("ui.msg_ejected", { path: at ?? deviceId }), "ok");
                break;
            }
            default:
                throw new Error(`unknown action ${act}`);
        }
        await invoke("refresh_menu");
        await loadVolumes();
        await loadDeps();
    } catch (err) {
        flashError(errText(act, err));
    } finally {
        if (button) {
            button.disabled = false;
            button.textContent = original;
        }
    }
}

// ---------------------------------------------------------------------------
// Modal (format confirmation)
// ---------------------------------------------------------------------------

const overlay = document.getElementById("modal-overlay");
const modalTitle = document.getElementById("modal-title");
const modalBody = document.getElementById("modal-body");
const modalConfirm = document.getElementById("modal-confirm");
let modalConfirmHandler = null;

function openModal(title, bodyHtml, confirmText, onConfirm) {
    modalTitle.textContent = title;
    modalBody.innerHTML = bodyHtml;
    modalConfirm.textContent = confirmText;
    modalConfirmHandler = onConfirm;
    overlay.hidden = false;
}

function closeModal() {
    overlay.hidden = true;
    modalConfirmHandler = null;
}

document.getElementById("modal-close").addEventListener("click", closeModal);
document.getElementById("modal-cancel").addEventListener("click", closeModal);
overlay.addEventListener("click", (ev) => {
    if (ev.target === overlay) {
        closeModal();
    }
});
modalConfirm.addEventListener("click", async () => {
    const handler = modalConfirmHandler;
    closeModal();
    if (typeof handler === "function") {
        await handler();
    }
});

function openFormatModal(deviceId) {
    const vol = volumes.find((v) => v.device_identifier === deviceId);
    if (!vol) {
        return;
    }
    openModal(
        tr("ui.format"),
        `<p class="warn">${esc(tr("ui.format_warn"))}</p>
         <dl class="facts">
             <dt>${esc(tr("ui.vol_dev"))}</dt><dd class="mono">${esc(vol.device_identifier)}</dd>
             <dt>${esc(tr("ui.vol_size"))}</dt><dd>${esc(vol.size_pretty)}</dd>
         </dl>
         <label class="field">
             <span>${esc(tr("ui.format_label"))}</span>
             <input id="format-label" type="text" maxlength="11" value="${esc(vol.volume_name || "")}">
         </label>`,
        tr("ui.format_confirm"),
        async () => {
            const label = document.getElementById("format-label");
            busy(true);
            try {
                await invoke("format_volume", {
                    deviceId,
                    label: label.value ? label.value : undefined,
                    quick: true,
                });
                toast(tr("ui.msg_format_ok"), "ok");
                await invoke("refresh_menu");
                await loadVolumes();
            } catch (err) {
                flashError(errText("format", err));
            } finally {
                busy(false);
            }
        }
    );
}

// ---------------------------------------------------------------------------
// Volume action delegation
// ---------------------------------------------------------------------------

document.getElementById("volumes-list").addEventListener("click", (ev) => {
    const button = ev.target.closest("button[data-act]");
    if (!button) {
        return;
    }
    const act = button.dataset.act;
    const deviceId = button.dataset.dev;
    if (act === "format") {
        openFormatModal(deviceId);
        return;
    }
    void runAction(act, deviceId, button);
});

// ---------------------------------------------------------------------------
// Header / toolbar
// ---------------------------------------------------------------------------

document.getElementById("lang-zh").addEventListener("click", () => void setLang("zh"));
document.getElementById("lang-en").addEventListener("click", () => void setLang("en"));

document.getElementById("refresh-btn").addEventListener("click", async () => {
    await Promise.all([loadDeps(), loadVolumes()]);
    await invoke("refresh_menu");
});

document.getElementById("install-btn").addEventListener("click", async () => {
    busy(true);
    try {
        const result = await invoke("install_dependencies");
        toast(String(result), "info", 8000);
    } catch (err) {
        flashError(tr("ui.msg_err_deps", { msg: err }));
    } finally {
        busy(false);
    }
});

// ---------------------------------------------------------------------------
// About block
// ---------------------------------------------------------------------------

async function loadAbout() {
    const el = document.getElementById("about-block");
    try {
        const info = await invoke("app_info");
        el.innerHTML = `<h2>${esc(tr("ui.about_title"))}</h2>
            <p class="about-lead">${esc(info.name)} · ${esc(tr("ui.about_version"))} ${esc(info.version)}</p>
            <p class="about-desc">${esc(info.description || "")}</p>
            <dl class="facts">
                <dt>${esc(tr("ui.about_repo"))}</dt><dd class="mono">${esc(info.repository || "")}</dd>
                <dt>${esc(tr("ui.about_license"))}</dt><dd>${esc(info.license || "")}</dd>
                <dt>${esc(tr("ui.about_authors"))}</dt><dd>${esc(info.authors || "")}</dd>
            </dl>`;
    } catch {
        el.innerHTML = `<h2>${esc(tr("ui.about_title"))}</h2>`;
    }
}

// ---------------------------------------------------------------------------
// Backend-pushed events
// ---------------------------------------------------------------------------

listen("ntfs-mac:refresh", () => {
    void loadDeps();
    void loadVolumes();
});

listen("ntfs-mac:error", (event) => {
    // Payload is `{ ok, message }`, sent for both menu-bar successes and
    // failures so the window can mirror a tray action's outcome.
    const payload = event && event.payload;
    const message = typeof payload === "string"
        ? payload
        : payload && payload.message
            ? payload.message
            : String(payload || tr("ui.no_deps"));
    toast(message, payload && payload.ok === false ? "error" : "ok");
});

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

applyI18n();
void loadDeps();
void loadVolumes();
void loadAbout();
