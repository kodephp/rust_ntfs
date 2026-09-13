// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 kodephp contributors
//
// Frontend contract checks for the ntfs-mac GUI. Run via scripts/test-frontend.sh.
//
// 1. Every `data-i18n` key in index.html resolves in both zh and en.
// 2. Every id referenced by `document.getElementById` in app.js exists in index.html.
// 3. The shared i18n keys/values in app.js match the Rust `Labels` constants
//    in lib.rs exactly, for both languages — the menu bar and the window can
//    never drift apart.
// 4. zh and en expose the same key set.
// 5. No stale references (`window.__TAURI__.core`, sponsor QR) remain.
// 6. Every backend call is bounded: no raw `invoke()`, and each command has an
//    explicit ceiling in `CALL_TIMEOUT_MS`.
// 7. UI ↔ API contract: the commands the window calls are exactly the commands
//    the backend registers — no dead handlers, no calls to commands that do
//    not exist.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "crates/ntfs-mac-tauri/src");

const html = readFileSync(join(src, "index.html"), "utf8");
const js = readFileSync(join(src, "app.js"), "utf8");
const rust = readFileSync(join(src, "../src-tauri/src/lib.rs"), "utf8");

let failures = 0;
const fail = (msg) => {
    failures += 1;
    console.error(`  ✗ ${msg}`);
};
const ok = (msg) => console.log(`  ✓ ${msg}`);

/** Extract the body of `const NAME = { ... };` by brace matching. */
function extractObject(source, name) {
    const start = source.indexOf(`const ${name} = {`);
    if (start < 0) return null;
    let depth = 0;
    let inString = false;
    let escaped = false;
    let quote = "";
    for (let i = start; i < source.length; i++) {
        const ch = source[i];
        if (inString) {
            if (escaped) {
                escaped = false;
            } else if (ch === "\\") {
                escaped = true;
            } else if (ch === quote) {
                inString = false;
            }
            continue;
        }
        if (ch === '"' || ch === "'" || ch === "`") {
            inString = true;
            quote = ch;
            continue;
        }
        if (ch === "{") depth++;
        else if (ch === "}") {
            depth--;
            if (depth === 0) return source.slice(start, i + 1);
        }
    }
    return null;
}

/** Pull the `key: "value"` pairs out of one Rust `Labels` literal. */
function extractRustLabels(source, constName) {
    const start = source.indexOf(`const ${constName}: Labels<'static>`);
    if (start < 0) return null;
    const brace = source.indexOf("{", start);
    const end = source.indexOf("};", brace);
    if (brace < 0 || end < 0) return null;
    const body = source.slice(brace + 1, end);
    const out = {};
    for (const match of body.matchAll(/^\s*([a-z_][a-z_0-9]*):\s*"((?:[^"\\]|\\.)*)"/gm)) {
        out[match[1]] = match[2].replace(/\\(?=\\)/g, "\\").replace(/\\"/g, '"');
    }
    return out;
}

console.log("ntfs-mac frontend contract");

// ---------------------------------------------------------------------------
// Parse both sides of the contract
// ---------------------------------------------------------------------------

const i18nBlock = extractObject(js, "I18N");
if (!i18nBlock) {
    console.error("FATAL: could not locate the I18N object in app.js");
    process.exit(1);
}
const I18N = new Function(
    `return ${i18nBlock.replace(/^const\s+I18N\s*=\s*/, "").replace(/;\s*$/, "")}`
)();

const zhLabels = extractRustLabels(rust, "ZH");
const enLabels = extractRustLabels(rust, "EN");
if (!zhLabels || !enLabels) {
    console.error("FATAL: could not locate the ZH / EN Labels constants in lib.rs");
    process.exit(1);
}

// ---------------------------------------------------------------------------
// 4. zh and en expose the same key set
// ---------------------------------------------------------------------------

for (const lang of ["zh", "en"]) {
    const other = lang === "zh" ? "en" : "zh";
    const keys = (o) => Object.keys(o).filter((k) => k !== "ui").sort().join(",");
    if (keys(I18N[lang]) !== keys(I18N[other])) {
        fail(`I18N ${lang} and ${other} do not expose the same shared key set`);
    }
    const uiKeys = (o) => Object.keys(o.ui).sort().join(",");
    if (uiKeys(I18N[lang]) !== uiKeys(I18N[other])) {
        fail(`I18N ${lang}.ui and ${other}.ui do not expose the same key set`);
    }
}
ok("zh / en dictionaries expose identical key sets");

// ---------------------------------------------------------------------------
// 3. Shared keys match the Rust Labels constants
// ---------------------------------------------------------------------------

for (const [lang, labels] of [["zh", zhLabels], ["en", enLabels]]) {
    const front = I18N[lang];
    const onlyRust = Object.keys(labels).filter((k) => !(k in front));
    const onlyFront = Object.keys(front)
        .filter((k) => k !== "ui" && !(k in labels));
    if (onlyRust.length) fail(`Labels.${lang} keys missing from app.js: ${onlyRust.join(", ")}`);
    if (onlyFront.length) fail(`app.js ${lang} keys not in Labels: ${onlyFront.join(", ")}`);
    for (const key of Object.keys(labels)) {
        if (front[key] !== labels[key]) {
            fail(`label drift ${lang}.${key}: rust="${labels[key]}" vs js="${front[key]}"`);
        }
    }
}
ok(`${Object.keys(zhLabels).length} shared labels match lib.rs exactly`);

// ---------------------------------------------------------------------------
// 1. Every data-i18n key resolves in both languages
// ---------------------------------------------------------------------------

function resolves(dict, key) {
    let node = dict;
    for (const part of key.split(".")) {
        if (node[part] === undefined) return false;
        node = node[part];
    }
    return true;
}

const i18nKeys = [...html.matchAll(/data-i18n="([^"]+)"/g)].map((m) => m[1]);
const unresolved = i18nKeys.filter((k) => !resolves(I18N.zh, k) || !resolves(I18N.en, k));
if (unresolved.length) {
    fail(`data-i18n keys with no translation: ${unresolved.join(", ")}`);
} else {
    ok(`${i18nKeys.length} data-i18n keys resolve in zh and en`);
}

// ---------------------------------------------------------------------------
// 2. Every getElementById target exists in the markup
// ---------------------------------------------------------------------------

const referenced = [...js.matchAll(/getElementById\("([^"]+)"\)/g)].map((m) => m[1]);
// Markup ids plus ids created dynamically by app.js (e.g. the format modal).
const declared = new Set([
    ...[...html.matchAll(/\bid="([^"]+)"/g)].map((m) => m[1]),
    ...[...js.matchAll(/\bid="([^"]+)"/g)].map((m) => m[1]),
]);
const missingIds = [...new Set(referenced)].filter((id) => !declared.has(id));
if (missingIds.length) {
    fail(`app.js references ids absent from index.html: ${missingIds.join(", ")}`);
} else {
    ok(`${new Set(referenced).size} element ids all present in index.html`);
}

// ---------------------------------------------------------------------------
// 5. Stale references and default-language checks
// ---------------------------------------------------------------------------

const stale = [
    [js, "window.__TAURI__.core", "app.js still uses the Tauri v1 invoke shim"],
    [js, "reveal_sponsor_qr", "app.js still calls the removed reveal_sponsor_qr command"],
    [html, "sponsor", "index.html still mentions the removed sponsor QR card"],
    [js, "sponsor", "app.js still mentions the removed sponsor QR card"],
];
for (const [haystack, needle, why] of stale) {
    if (haystack.includes(needle)) fail(why);
}

if (!/<html lang="zh">/.test(html)) {
    fail('index.html must default to lang="zh"');
}

const legacyEnglish = [
    "System Status",
    "NTFS Volumes",
    "No NTFS volumes found",
    "Support This Project",
    "Install Missing Dependencies",
    "Loading volumes",
    "Checking dependencies",
    ">Refresh<",
];
const foundLegacy = legacyEnglish.filter((s) => html.includes(s));
if (foundLegacy.length) {
    fail(`index.html still shows legacy English copy: ${foundLegacy.join(", ")}`);
}
ok("no stale references; index.html defaults to Chinese");

// ---------------------------------------------------------------------------
// 6. Every backend call is bounded by the timeout wrapper
//
// A raw invoke() with a literal command name is a regression: it means a call
// can hang the UI with nothing to time it out.
// ---------------------------------------------------------------------------

if (/invoke\("/.test(js)) {
    fail("app.js has a raw invoke() with a literal command name; wrap it in call()");
}

const timeoutTable = extractObject(js, "CALL_TIMEOUT_MS");
if (!timeoutTable) {
    fail("app.js no longer defines CALL_TIMEOUT_MS — every call needs a ceiling");
} else {
    const listed = new Set(
        [...timeoutTable.matchAll(/^\s{4}([a-z_][a-z_0-9]*):\s*\d/mg)].map((m) => m[1])
    );
    const called = new Set(
        [...js.matchAll(/call\("([a-z_][a-z_0-9]*)"/g)].map((m) => m[1])
    );
    const untimed = [...called].filter((c) => !listed.has(c));
    if (untimed.length) {
        fail(`commands with no explicit timeout ceiling: ${untimed.join(", ")}`);
    } else {
        ok(`all ${called.size} backend calls are bounded by CALL_TIMEOUT_MS`);
    }
}

// ---------------------------------------------------------------------------
// 7. UI ↔ API contract: window and backend agree on the command set
// ---------------------------------------------------------------------------

const handlerBlock = rust.match(/tauri::generate_handler!\[([\s\S]*?)\]/);
if (!handlerBlock) {
    console.error("FATAL: could not locate generate_handler! in lib.rs");
    process.exit(1);
}
const registered = [...handlerBlock[1].matchAll(/\b([a-z_][a-z_0-9]*)\b/g)].map((m) => m[1]);
const usedByUi = new Set(
    [...js.matchAll(/call\("([a-z_][a-z_0-9]*)"/g)].map((m) => m[1])
);

const deadCommands = registered.filter((c) => !usedByUi.has(c));
const ghostCommands = [...usedByUi].filter((c) => !registered.includes(c));
if (deadCommands.length) {
    fail(`registered but never called by the window: ${deadCommands.join(", ")}`);
}
if (ghostCommands.length) {
    fail(`called by the window but not registered: ${ghostCommands.join(", ")}`);
}
if (!deadCommands.length && !ghostCommands.length) {
    ok(`${registered.length} commands: UI and backend agree exactly`);
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

if (failures) {
    console.error(`\n${failures} frontend contract check(s) failed`);
    process.exit(1);
}
console.log("\nfrontend contract: all checks passed");
