export function createUpdateFeature(deps) {
  const {
    invoke,
    appState,
    getCuratedData,
    modListContainer,
    updateListContainer,
    updateModalOverlay,
    i18n,
    iconNexus,
    check,
    relaunch,
    resolveUpdateCandidate,
    getBaseName,
    isNewerVersionAvailable,
  } = deps;

  async function checkAppUpdate(isManual = false) {
    try {
      const isInstalled = await invoke('is_app_installed');

      if (!isInstalled) {
        console.log('Portable mode detected. Auto-updater disabled.');
        if (isManual) {
          await window.customAlert(i18n.get('portableModeMsg'), i18n.get('portableModeTitle'));
        }
        return;
      }

      if (isManual) {
        const btn = document.getElementById('checkAppUpdateBtn');
        if (btn) {
          btn.textContent = i18n.get('statusChecking');
          btn.disabled = true;
        }
      } else {
        console.log('Running silent startup app update check...');
      }

      const update = await check();
      if (!update) {
        if (isManual) {
          await window.customAlert(i18n.get('updateUpToDateMsg'), i18n.get('updateUpToDateTitle'));
        }
        return;
      }

      const confirmed = await window.customConfirm(
        i18n.get('appUpdateMsg', { version: update.version, notes: update.body || '' }),
        i18n.get('appUpdateAvailableTitle'),
        i18n.get('btnUpdateRestart'),
        i18n.get('btnLater')
      );

      if (!confirmed) return;

      await window.customAlert(i18n.get('statusDownloadingUpdate'), i18n.get('statusUpdating'));

      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          console.log(`Update started: ${event.data.contentLength} bytes`);
        } else if (event.event === 'Progress') {
          console.log(`Update progress: ${event.data.chunkLength}`);
        } else if (event.event === 'Finished') {
          console.log('Update download finished.');
        }
      });

      await relaunch();
    } catch (error) {
      console.error('App update check failed:', error);
      if (isManual) {
        await window.customAlert(i18n.get('updateErrorMsg', { error: String(error) }), i18n.get('updateErrorTitle'));
      }
    } finally {
      if (isManual) {
        const btn = document.getElementById('checkAppUpdateBtn');
        if (btn) {
          btn.textContent = i18n.get('checkUpdateBtn');
          btn.disabled = false;
        }
      }
    }
  }

  async function checkForUpdates(isSilent = false) {
    const curatedData = getCuratedData();
    if (!appState.gamePath || curatedData.length === 0) {
      if (!isSilent) {
        await window.customAlert('Mod data is not loaded. Cannot check for updates.', 'Error');
      }
      return;
    }

    if (isSilent) {
      console.log('Performing silent update check...');
    } else {
      updateListContainer.innerHTML = `<p>${i18n.get('updateChecking')}</p>`;
      updateModalOverlay.classList.remove('hidden');
    }

    const groupedUpdates = new Map();

    for (const [modFolderName, cachedModData] of appState.modDataCache.entries()) {
      const updateCandidate = resolveUpdateCandidate(
        modFolderName,
        cachedModData,
        curatedData,
        { getBaseName, isNewerVersionAvailable }
      );
      if (!updateCandidate) continue;

      const row = modListContainer.querySelector(`.mod-row[data-mod-name="${modFolderName}"]`);
      const indicator = row?.querySelector('.update-indicator');
      if (indicator) indicator.classList.remove('hidden');

      if (!groupedUpdates.has(updateCandidate.modId)) {
        groupedUpdates.set(updateCandidate.modId, {
          name: updateCandidate.modName,
          installed: updateCandidate.installedVersion,
          latest: updateCandidate.latestVersion,
          nexusUrl: updateCandidate.nexusUrl,
          modId: updateCandidate.modId,
          fileId: updateCandidate.latestFileId,
          fileName: updateCandidate.latestFileName,
          displayName: updateCandidate.latestDisplayName,
          replacingFileId: updateCandidate.installedFileId,
          folders: [modFolderName],
        });
      } else {
        groupedUpdates.get(updateCandidate.modId).folders.push(modFolderName);
      }
    }

    if (isSilent) return;

    updateListContainer.innerHTML = '';
    if (groupedUpdates.size === 0) {
      updateListContainer.innerHTML = `<p>${i18n.get('updateNoneFound')}</p>`;
      return;
    }

    groupedUpdates.forEach((updateInfo) => {
      const item = document.createElement('div');
      item.className = 'update-item';

      const nexusLinkHtml = updateInfo.nexusUrl
        ? `<a href="${updateInfo.nexusUrl}" class="nexus-button" target="_blank" title="${i18n.get('btnVisitNexus')}"><img src="${iconNexus}" alt="Nexus"></a>`
        : '';

      const folderListStr = updateInfo.folders.join(', ');
      const folderCountText = updateInfo.folders.length > 1
        ? `<div style="font-size: 0.85em; opacity: 0.7; margin-top: 4px;">Affects ${updateInfo.folders.length} folders: ${folderListStr}</div>`
        : '';

      item.innerHTML = `
        <div class="update-item-info">
          <div class="update-item-name">${updateInfo.name}</div>
          <div class="update-item-version">
            ${updateInfo.installed} <span class="arrow">→</span> <span class="latest">${updateInfo.latest}</span>
          </div>
          ${folderCountText}
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          ${nexusLinkHtml}
          <button class="modal-btn-confirm update-now-btn"
            data-mod-id="${updateInfo.modId}"
            data-file-id="${updateInfo.fileId}"
            data-version="${updateInfo.latest}"
            data-file-name="${updateInfo.fileName}"
            data-display-name="${updateInfo.displayName}"
            data-replacing-file-id="${updateInfo.replacingFileId || ''}">UPDATE</button>
        </div>`;

      updateListContainer.appendChild(item);
    });
  }

  return {
    checkAppUpdate,
    checkForUpdates,
  };
}
