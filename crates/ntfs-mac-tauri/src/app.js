const { invoke } = window.__TAURI__.core;

let volumes = [];

async function loadDeps() {
    const el = document.getElementById('deps-status');
    el.innerHTML = '<div class="loading">Checking dependencies...</div>';
    try {
        const report = await invoke('check_deps');
        document.getElementById('platform-info').textContent = `macOS ${report.macos_version || '?'} on ${report.arch}`;
        let html = '';
        for (const dep of report.deps) {
            const icon = dep.present ? '✓' : '✗';
            const cls = dep.present ? 'dep-check' : 'dep-missing';
            const path = dep.path ? `<span class="dep-path">(${dep.path})</span>` : '';
            const hint = !dep.present && dep.install_hint ? `<br><span class="dep-path">${dep.install_hint}</span>` : '';
            html += `<div class="dep-item"><span class="${cls}">${icon}</span><span class="dep-name">${dep.name}</span>${path}${hint}</div>`;
        }
        el.innerHTML = html;
        const installBtn = document.getElementById('install-btn');
        installBtn.style.display = report.ready ? 'none' : 'inline-block';
    } catch (e) {
        el.innerHTML = `<div class="toast-error">Error: ${e}</div>`;
    }
}

async function loadVolumes() {
    const el = document.getElementById('volumes-list');
    el.innerHTML = '<div class="loading">Loading volumes...</div>';
    try {
        volumes = await invoke('list_volumes');
        if (volumes.length === 0) {
            el.innerHTML = '<div style="text-align:center;padding:20px;color:var(--text-muted)">No NTFS volumes found</div>';
            return;
        }
        let html = '';
        for (const v of volumes) {
            const icon = v.mounted ? '💾' : '📁';
            const badge = v.mounted ? '<span class="badge badge-mounted">mounted</span>' : '<span class="badge badge-unmounted">unmounted</span>';
            const mountInfo = v.mount_point ? ` at ${v.mount_point}` : '';
            html += `
                <div class="volume-item" data-device="${v.device_identifier}">
                    <span class="volume-icon">${icon}</span>
                    <div class="volume-info">
                        <div class="volume-name">${v.display_label}</div>
                        <div class="volume-details">${v.size_pretty} · ${v.media_type}${mountInfo}</div>
                    </div>
                    <div class="volume-actions">
                        ${v.mounted
                            ? `<button class="btn btn-small btn-secondary" onclick="unmount('${v.device_identifier}')">Unmount</button>`
                            : `<button class="btn btn-small btn-success" onclick="mount('${v.device_identifier}')">Mount</button>
                               <button class="btn btn-small btn-danger" onclick="showFormatModal('${v.device_identifier}')">Format</button>`}
                        <button class="btn btn-small btn-secondary" onclick="fixVolume('${v.device_identifier}')">Fix</button>
                    </div>
                </div>`;
        }
        el.innerHTML = html;
    } catch (e) {
        el.innerHTML = `<div class="toast-error">Error: ${e}</div>`;
    }
}

async function mount(deviceId) {
    try {
        const mountPoint = await invoke('mount_volume', { deviceId, readonly: false });
        showToast(`Mounted at ${mountPoint}`, 'success');
        loadVolumes();
    } catch (e) {
        showToast(`Mount failed: ${e}`, 'error');
    }
}

async function unmount(deviceId) {
    try {
        await invoke('unmount_volume', { deviceId });
        showToast(`Unmounted ${deviceId}`, 'success');
        loadVolumes();
    } catch (e) {
        showToast(`Unmount failed: ${e}`, 'error');
    }
}

function showFormatModal(deviceId) {
    const vol = volumes.find(v => v.device_identifier === deviceId);
    if (!vol) return;
    document.getElementById('modal-title').textContent = 'Format Volume';
    document.getElementById('modal-body').innerHTML = `
        <p style="color:var(--danger);margin-bottom:12px">⚠️ This will ERASE all data on the volume!</p>
        <p><strong>Volume:</strong> ${vol.display_label}</p>
        <p><strong>Size:</strong> ${vol.size_pretty}</p>
        <label style="display:block;margin-top:12px">Volume Label (max 11 chars):</label>
        <input type="text" id="format-label" maxlength="11" style="width:100%;padding:8px;margin-top:4px;background:var(--bg);border:1px solid var(--border);border-radius:4px;color:var(--text)">
    `;
    document.getElementById('modal-overlay').style.display = 'flex';
    document.getElementById('modal-confirm').onclick = () => doFormat(deviceId);
}

async function doFormat(deviceId) {
    const label = document.getElementById('format-label').value || undefined;
    document.getElementById('modal-overlay').style.display = 'none';
    try {
        await invoke('format_volume', { deviceId, label, quick: true });
        showToast('Volume formatted', 'success');
        loadVolumes();
    } catch (e) {
        showToast(`Format failed: ${e}`, 'error');
    }
}

async function fixVolume(deviceId) {
    try {
        await invoke('fix_volume', { deviceId, useFsck: false });
        showToast(`Fixed ${deviceId}`, 'success');
    } catch (e) {
        showToast(`Fix failed: ${e}`, 'error');
    }
}

function showToast(message, type = 'info') {
    const container = document.getElementById('toast-container');
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => toast.remove(), 4000);
}

document.getElementById('refresh-btn').onclick = () => { loadDeps(); loadVolumes(); };
document.getElementById('install-btn').onclick = () => {
    showToast('Run ./scripts/install.sh in terminal to install dependencies', 'info');
};

/* Sponsor QR code actions */
document.getElementById('sponsor-copy-btn').onclick = async () => {
    try {
        await navigator.clipboard.writeText('src/assets/sponsor-qr.svg');
        showToast('QR code path copied', 'success');
    } catch (e) {
        showToast('Copy failed', 'error');
    }
};
document.getElementById('sponsor-open-btn').onclick = async () => {
    try {
        const path = await invoke('reveal_sponsor_qr');
        showToast(`QR code: ${path}`, 'success');
    } catch (e) {
        showToast(`Reveal failed: ${e}`, 'error');
    }
};
document.getElementById('sponsor-qr-img').onclick = async () => {
    try {
        await invoke('reveal_sponsor_qr');
        showToast('QR code revealed in Finder', 'success');
    } catch (e) {
        showToast(`Failed: ${e}`, 'error');
    }
};

const modal = document.getElementById('modal-overlay');
modal.querySelector('.modal-close').onclick = () => modal.style.display = 'none';
document.getElementById('modal-cancel').onclick = () => modal.style.display = 'none';

loadDeps();
loadVolumes();
