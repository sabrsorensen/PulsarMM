export function createSettingsHelpers(deps) {
  const {
    invoke,
    i18n,
    currentDownloadPathEl,
    currentLibraryPathEl,
  } = deps;

  async function updateDownloadPathUI() {
    try {
      const path = await invoke('get_downloads_path');
      currentDownloadPathEl.textContent = path;
    } catch {
      currentDownloadPathEl.textContent = 'Error loading path';
    }
  }

  async function updateLibraryPathUI() {
    try {
      const path = await invoke('get_library_path');
      currentLibraryPathEl.textContent = path;
    } catch {
      currentLibraryPathEl.textContent = 'Error loading path';
    }
  }

  async function updateNXMButtonState() {
    try {
      const isRegistered = await invoke('is_protocol_handler_registered');
      const btn = document.getElementById('nxmHandlerBtn');
      const statusEl = document.getElementById('nxmHandlerStatus');

      if (isRegistered) {
        btn.textContent = i18n.get('removeHandlerBtn');
        btn.className = 'modal-btn-nxm';
        btn.classList.remove('modal-btn-nxm-confirm');
        if (statusEl) statusEl.classList.add('hidden');
      } else {
        btn.textContent = i18n.get('setHandlerBtn');
        btn.className = 'modal-btn-nxm-confirm';
        btn.classList.remove('modal-btn-nxm');
        if (statusEl) statusEl.classList.add('hidden');
      }
    } catch (e) {
      console.warn('NXM check failed', e);
    }
  }

  return {
    updateDownloadPathUI,
    updateLibraryPathUI,
    updateNXMButtonState,
  };
}
