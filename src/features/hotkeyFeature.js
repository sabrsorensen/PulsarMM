export function installGlobalHotkeys(deps) {
  const {
    getInputMode,
    appState,
    modListContainer,
    modInfoPanel,
    downloadListContainer,
  } = deps;

  window.addEventListener('keydown', (e) => {
    if (e.key === 'Tab') {
      if (getInputMode() !== 'gamepad') {
        e.preventDefault();
      }
      return;
    }

    if (e.key === 'Escape') {
      if (!document.getElementById('inputDialogModal').classList.contains('hidden')) {
        document.getElementById('inputDialogCancelBtn').click();
        return;
      }
      if (!document.getElementById('genericDialogModal').classList.contains('hidden')) {
        document.querySelector('#genericDialogActions button')?.click();
        return;
      }

      const modals = [
        'profileProgressModal', 'profileManagerModal', 'folderSelectionModal',
        'downloadHistoryModalOverlay', 'fileSelectionModalOverlay',
        'updateModalOverlay', 'priorityModalOverlay', 'changelogModalOverlay',
        'settingsModalOverlay', 'modDetailPanel',
      ];

      for (const id of modals) {
        const el = document.getElementById(id);
        if (el && !el.classList.contains('hidden')) {
          if (id === 'modDetailPanel' && el.classList.contains('open')) {
            document.getElementById('modDetailCloseBtn').click();
            return;
          }
          if (id === 'downloadHistoryModalOverlay') {
            appState.selectedDownloadIds.clear();
          }
          if (id !== 'modDetailPanel') {
            el.classList.add('hidden');
            return;
          }
        }
      }

      if (appState.selectedModNames.size > 0) {
        appState.selectedModNames.clear();
        appState.selectedModRow = null;
        modListContainer.querySelectorAll('.mod-row.selected').forEach(el => el.classList.remove('selected'));
        modInfoPanel.classList.add('hidden');
      }

      if (appState.selectedDownloadIds.size > 0) {
        appState.selectedDownloadIds.clear();
        downloadListContainer.querySelectorAll('.download-item.selected').forEach(el => el.classList.remove('selected'));
      }
    }

    if (e.key === 'Enter') {
      if (!document.getElementById('inputDialogModal').classList.contains('hidden')) {
        document.getElementById('inputDialogOkBtn').click();
        return;
      }
      if (!document.getElementById('genericDialogModal').classList.contains('hidden')) {
        document.querySelector('.modal-gen-btn-confirm')?.click();
      }
    }
  });
}
