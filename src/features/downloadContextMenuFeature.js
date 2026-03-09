export function createDownloadContextMenuFeature(deps) {
  const {
    appState,
    downloadListContainer,
    i18n,
    invoke,
    getDownloadHistory,
    renderDownloadHistory,
    saveDownloadHistory,
    handleDownloadItemInstall,
    removeContextMenu,
    setContextMenu,
  } = deps;

  function ensureSingleDownloadSelection(downloadId) {
    if (appState.selectedDownloadIds.has(downloadId)) {
      return;
    }

    appState.selectedDownloadIds.clear();
    appState.selectedDownloadIds.add(downloadId);

    const allRows = downloadListContainer.querySelectorAll('.download-item');
    allRows.forEach(row => {
      if (row.dataset.downloadId === downloadId) {
        row.classList.add('selected');
      } else {
        row.classList.remove('selected');
      }
    });
  }

  async function handleDownloadItemDelete(downloadId) {
    const confirmed = await window.customConfirm(
      i18n.get('deleteDownloadArchiveMsg'),
      i18n.get('deleteDownloadArchiveTitle')
    );
    if (!confirmed) return;

    const downloadHistory = getDownloadHistory();
    const itemIndex = downloadHistory.findIndex(d => d.id === downloadId);
    if (itemIndex === -1) return;

    const item = downloadHistory[itemIndex];

    try {
      if (item.archivePath) {
        await invoke('delete_archive_file', { path: item.archivePath });
      }

      if (item.fileName) {
        await invoke('delete_library_folder', { zipFilename: item.fileName });
      }

      downloadHistory.splice(itemIndex, 1);
      renderDownloadHistory();
      await saveDownloadHistory(downloadHistory);
    } catch (error) {
      await window.customAlert(`Failed to delete files: ${error}`, 'Error');
    }
  }

  async function handleBulkDelete() {
    const selectionCount = appState.selectedDownloadIds.size;
    const confirmed = await window.customConfirm(
      i18n.get('confirmDeleteMultipleMsg') || 'Delete selected files?',
      i18n.get('confirmDeleteTitle') || 'Confirm Delete'
    );
    if (!confirmed) return;

    const downloadHistory = getDownloadHistory();
    const idsToDelete = Array.from(appState.selectedDownloadIds);

    for (const id of idsToDelete) {
      const itemIndex = downloadHistory.findIndex(d => d.id === id);
      if (itemIndex === -1) continue;

      const item = downloadHistory[itemIndex];
      try {
        if (item.archivePath) {
          await invoke('delete_archive_file', { path: item.archivePath });
        }
        if (item.fileName) {
          await invoke('delete_library_folder', { zipFilename: item.fileName });
        }
        downloadHistory.splice(itemIndex, 1);
      } catch (err) {
        console.error(`Failed to delete ${item.fileName}`, err);
      }
    }

    if (selectionCount > 0) {
      appState.selectedDownloadIds.clear();
      renderDownloadHistory();
      await saveDownloadHistory(downloadHistory);
    }
  }

  function showDownloadContextMenu(e, downloadId) {
    e.preventDefault();
    e.stopPropagation();
    removeContextMenu();

    ensureSingleDownloadSelection(downloadId);

    const contextMenu = document.createElement('div');
    contextMenu.className = 'context-menu';
    contextMenu.style.left = `${e.clientX}px`;
    contextMenu.style.top = `${e.clientY}px`;

    const selectionCount = appState.selectedDownloadIds.size;
    if (selectionCount > 1) {
      const deleteButton = document.createElement('button');
      deleteButton.textContent = i18n.get('deleteMultipleBtn', { count: selectionCount }) || `Delete ${selectionCount} Items`;
      deleteButton.className = 'context-menu-item delete';
      deleteButton.onclick = async () => {
        removeContextMenu();
        await handleBulkDelete();
      };
      contextMenu.appendChild(deleteButton);
      setContextMenu(contextMenu);
      return;
    }

    const downloadHistory = getDownloadHistory();
    const itemData = downloadHistory.find(d => d.id === downloadId);
    if (!itemData) return;

    if (
      (itemData.statusClass === 'success' ||
        itemData.statusClass === 'cancelled' ||
        itemData.statusClass === 'error' ||
        itemData.statusClass === 'unpacked') &&
      itemData.archivePath
    ) {
      const installButton = document.createElement('button');
      installButton.textContent = i18n.get('ctxInstall');
      installButton.className = 'context-menu-item';
      installButton.onclick = () => handleDownloadItemInstall(downloadId);
      contextMenu.appendChild(installButton);
    }

    const nexusButton = document.createElement('button');
    nexusButton.textContent = i18n.get('ctxVisitNexus');
    nexusButton.className = 'context-menu-item';
    nexusButton.onclick = () => {
      invoke('plugin:shell|open', {
        path: `https://www.nexusmods.com/nomanssky/mods/${itemData.modId}`,
        with: null,
      });
    };
    contextMenu.appendChild(nexusButton);

    if (itemData.archivePath) {
      const revealButton = document.createElement('button');
      revealButton.textContent = i18n.get('ctxRevealExplorer');
      revealButton.className = 'context-menu-item';
      revealButton.onclick = () => invoke('show_in_folder', { path: itemData.archivePath });
      contextMenu.appendChild(revealButton);
    }

    const deleteButton = document.createElement('button');
    deleteButton.textContent = i18n.get('deleteBtn');
    deleteButton.className = 'context-menu-item delete';
    deleteButton.onclick = () => handleDownloadItemDelete(downloadId);
    contextMenu.appendChild(deleteButton);

    setContextMenu(contextMenu);
  }

  return {
    showDownloadContextMenu,
    handleDownloadItemDelete,
  };
}
