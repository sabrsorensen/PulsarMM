export function createDownloadHistoryFeature(deps) {
  const {
    appDataDir,
    join,
    readTextFile,
    writeTextFile,
    mkdir,
    formatBytes,
    formatDate,
    i18n,
    appState,
    downloadSortState,
    nexusApi,
    getCuratedData,
    getDownloadHistory,
    setDownloadHistory,
    onInstall,
    onContextMenu,
    onStartModDownload,
    elements,
  } = deps;

  const {
    downloadListContainer,
    browseGridContainer,
    modDetailPanel,
    modDetailName,
    modDetailInstallBtnContainer,
    modDetailInstalled,
  } = elements;

  async function loadDownloadHistory() {
    try {
      const dataDir = await appDataDir();
      const historyFilePath = await join(dataDir, 'download_history.json');
      const content = await readTextFile(historyFilePath);
      setDownloadHistory(JSON.parse(content));
    } catch (e) {
      console.log('No download history file found. Starting fresh.');
      setDownloadHistory([]);
    }
  }

  async function saveDownloadHistory(history) {
    try {
      const dataDir = await appDataDir();
      await mkdir(dataDir, { recursive: true });
      const historyFilePath = await join(dataDir, 'download_history.json');
      await writeTextFile(historyFilePath, JSON.stringify(history, null, 2));
    } catch (error) {
      console.error('Failed to save download history:', error);
    }
  }

  function renderDownloadHistory() {
    const downloadHistory = getDownloadHistory();

    downloadHistory.sort((a, b) => {
      let valA;
      let valB;
      switch (downloadSortState.key) {
        case 'name':
          valA = (a.displayName || a.fileName || '').toLowerCase();
          valB = (b.displayName || b.fileName || '').toLowerCase();
          break;
        case 'status':
          valA = (a.statusText || '').toLowerCase();
          valB = (b.statusText || '').toLowerCase();
          break;
        case 'size':
          valA = a.size || 0;
          valB = b.size || 0;
          break;
        default:
          valA = a.createdAt || parseInt(a.id.split('-')[1], 10);
          valB = b.createdAt || parseInt(b.id.split('-')[1], 10);
          break;
      }

      if (valA < valB) return downloadSortState.direction === 'asc' ? -1 : 1;
      if (valA > valB) return downloadSortState.direction === 'asc' ? 1 : -1;
      return 0;
    });

    downloadListContainer.innerHTML = '';
    const template = document.getElementById('downloadItemTemplate');

    for (const itemData of downloadHistory) {
      const newItem = template.content.cloneNode(true).firstElementChild;

      const progressBar = document.createElement('div');
      progressBar.className = 'download-progress-bar';
      newItem.appendChild(progressBar);

      newItem.dataset.downloadId = itemData.id;

      if (appState.selectedDownloadIds.has(itemData.id)) {
        newItem.classList.add('selected');
      }

      if ((itemData.statusClass === 'success' ||
        itemData.statusClass === 'cancelled' ||
        itemData.statusClass === 'error' ||
        itemData.statusClass === 'unpacked') && itemData.archivePath) {
        newItem.classList.add('installable');
      }

      const displayName = (itemData.displayName && itemData.version)
        ? `${itemData.displayName} (${itemData.version})`
        : itemData.fileName;
      const nameEl = newItem.querySelector('.download-item-name');
      nameEl.textContent = displayName;
      nameEl.setAttribute('title', displayName);

      const statusEl = newItem.querySelector('.download-item-status');

      statusEl.className = 'download-item-status';
      statusEl.classList.add(`status-${itemData.statusClass}`);

      if (itemData.statusClass === 'installed') {
        statusEl.textContent = i18n.get('statusInstalled');
      } else if (itemData.statusClass === 'success') {
        statusEl.textContent = i18n.get('statusDownloaded');
      } else if (itemData.statusClass === 'unpacked') {
        statusEl.textContent = i18n.get('statusUnpacked');
      } else if (itemData.statusClass === 'cancelled') {
        statusEl.textContent = i18n.get('statusCancelled');
      } else {
        statusEl.textContent = itemData.statusText;
      }

      newItem.querySelector('.download-item-size').textContent = formatBytes(itemData.size);

      const timestamp = itemData.createdAt || parseInt(itemData.id.split('-')[1], 10) / 1000;
      newItem.querySelector('.download-item-date').textContent = formatDate(timestamp);

      newItem.addEventListener('click', (e) => {
        if (e.ctrlKey) {
          if (appState.selectedDownloadIds.has(itemData.id)) {
            appState.selectedDownloadIds.delete(itemData.id);
            newItem.classList.remove('selected');
          } else {
            appState.selectedDownloadIds.add(itemData.id);
            newItem.classList.add('selected');
          }
        } else {
          appState.selectedDownloadIds.clear();
          appState.selectedDownloadIds.add(itemData.id);

          const allRows = downloadListContainer.querySelectorAll('.download-item');
          allRows.forEach(row => row.classList.remove('selected'));
          newItem.classList.add('selected');
        }
      });

      newItem.addEventListener('dblclick', () => {
        if (newItem.classList.contains('installable')) {
          onInstall(itemData.id);
        }
      });

      newItem.addEventListener('contextmenu', (e) => onContextMenu(e, itemData.id));

      downloadListContainer.appendChild(newItem);
    }

    const headerRow = document.querySelector('.download-header-row');
    if (headerRow) {
      headerRow.querySelectorAll('.sortable').forEach(header => {
        header.classList.remove('asc', 'desc');
        if (header.dataset.sort === downloadSortState.key) {
          header.classList.add(downloadSortState.direction);
        }
      });
    }
  }

  function updateDownloadStatus(downloadId, text, statusClass, modName) {
    const item = document.getElementById(downloadId);
    if (!item) return;

    const nameEl = item.querySelector('.download-item-name');
    const statusEl = item.querySelector('.download-item-status');

    if (modName && nameEl) {
      nameEl.textContent = modName;
    }

    if (statusEl) {
      statusEl.textContent = text;
      statusEl.className = 'download-item-status';
      statusEl.classList.add(`status-${statusClass}`);
    }
  }

  async function handleNxmLink(link) {
    console.log(`Frontend received nxm link: ${link}`);

    const match = link.match(/nxm:\/\/nomanssky\/mods\/(\d+)\/files\/(\d+)/);
    if (!match || match.length < 3) {
      await window.customAlert('Error: The received Nexus link was malformed.', 'Link Error');
      return;
    }

    const modId = match[1];
    const fileId = match[2];

    const queryParts = link.split('?');
    const queryParams = queryParts.length > 1 ? queryParts[1] : '';

    let fileInfo = null;

    const curatedData = getCuratedData();
    const localMod = curatedData.find(m => String(m.mod_id) === modId);
    if (localMod && localMod.files) {
      fileInfo = localMod.files.find(f => String(f.file_id) === fileId);
      if (fileInfo) {
        console.log('NXM Link: Found file info in local cache.');
      }
    }

    if (!fileInfo) {
      console.log('NXM Link: Mod not in local cache. Fetching from API...');
      const filesData = await nexusApi.fetchModFilesFromNexus(modId);
      if (filesData && filesData.files) {
        fileInfo = filesData.files.find(f => String(f.file_id) === fileId);
      }
    }

    if (!fileInfo) {
      await window.customAlert(`File ID ${fileId} not found for mod ${modId}.`, 'Error');
      return;
    }

    const displayName = fileInfo.name || fileInfo.file_name;

    await onStartModDownload({
      modId,
      fileId,
      version: fileInfo.version,
      fileName: fileInfo.file_name,
      displayName,
      replacingFileId: null,
      nxmQueryParams: queryParams,
    });
  }

  function updateModDisplayState(modId) {
    const modIdStr = String(modId);
    const installedFiles = appState.installedModsMap.get(modIdStr);
    const isInstalled = installedFiles && installedFiles.size > 0;

    if (!modDetailPanel.classList.contains('hidden') && modDetailName.dataset.modId === modIdStr) {
      const primaryBtn = modDetailInstallBtnContainer.querySelector('.mod-card-install-btn');
      if (primaryBtn) {
        primaryBtn.textContent = isInstalled ? 'MANAGE FILES' : 'DOWNLOAD';
      }

      if (isInstalled) {
        modDetailInstalled.textContent = installedFiles.values().next().value || 'Installed';
      } else {
        modDetailInstalled.textContent = 'N/A';
      }
    }

    const card = browseGridContainer.querySelector(`.mod-card[data-mod-id="${modIdStr}"]`);
    if (card) {
      const badge = card.querySelector('.mod-card-installed-badge');

      card.classList.toggle('is-installed', isInstalled);
      if (badge) {
        badge.classList.toggle('hidden', !isInstalled);
      }
    }
  }

  return {
    loadDownloadHistory,
    saveDownloadHistory,
    renderDownloadHistory,
    updateDownloadStatus,
    handleNxmLink,
    updateModDisplayState,
  };
}
