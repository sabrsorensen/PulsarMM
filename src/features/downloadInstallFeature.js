export function createDownloadInstallFeature(deps) {
  const {
    i18n,
    invoke,
    join,
    readTextFile,
    appState,
    nexusApi,
    downloadHistoryModalOverlay,
    getDownloadHistory,
    setDownloadHistory,
    renderDownloadHistory,
    saveDownloadHistory,
    openFolderSelectionModal,
    loadXmlContent,
    renderModList,
    addNewModToXml,
    checkForAndLinkMod,
    saveChanges,
    saveCurrentProfile,
    updateModDisplayState,
    logInfo,
    logError,
  } = deps;

  function updateDownloadItemStatus(downloadId, text, statusClass) {
    const downloadHistory = getDownloadHistory();
    const currentItem = downloadHistory.find(d => d.id === downloadId);
    if (currentItem) {
      currentItem.statusText = text;
      currentItem.statusClass = statusClass;
      renderDownloadHistory();
    }
  }

  async function processInstallAnalysis(analysis, item, isUpdate) {
    const downloadHistory = getDownloadHistory();

    if (analysis.conflicts && analysis.conflicts.length > 0) {
      for (const conflict of analysis.conflicts) {
        const oldItemIndex = downloadHistory.findIndex(d => d.modFolderName === conflict.old_mod_folder_name);
        if (oldItemIndex > -1) {
          const oldItem = downloadHistory[oldItemIndex];
          if (oldItem.archivePath && oldItem.id !== item.id) {
            downloadHistory.splice(oldItemIndex, 1);
          }
        }

        const shouldReplace = isUpdate ? true : await window.customConfirm(
          i18n.get('modUpdateConflictMsg', {
            oldName: conflict.old_mod_folder_name,
            newName: conflict.new_mod_name,
          }),
          i18n.get('modUpdateConflictTitle')
        );

        await invoke('resolve_conflict', {
          newModName: conflict.new_mod_name,
          oldModFolderName: conflict.old_mod_folder_name,
          tempModPathStr: conflict.temp_path,
          replace: shouldReplace,
        });

        if (shouldReplace) {
          if (conflict.new_mod_name.toUpperCase() !== conflict.old_mod_folder_name.toUpperCase()) {
            const updatedXmlContent = await invoke('update_mod_name_in_xml', {
              oldName: conflict.old_mod_folder_name.toUpperCase(),
              newName: conflict.new_mod_name.toUpperCase(),
            });
            await loadXmlContent(updatedXmlContent, appState.currentFilePath);
          }

          await invoke('ensure_mod_info', {
            modFolderName: conflict.new_mod_name,
            modId: item.modId || '',
            fileId: item.fileId || '',
            version: item.version || '',
            installSource: item.fileName,
          });

          item.modFolderName = conflict.new_mod_name;
        }
      }
    }

    if (analysis.successes && analysis.successes.length > 0) {
      const processedNames = new Set();
      let hasPromptedForRename = false;

      for (const mod of analysis.successes) {
        if (processedNames.has(mod.name)) continue;

        let isRenamedEntry = false;
        let oldFolderName = null;

        if (item.modId && !hasPromptedForRename) {
          let bestMatch = null;
          let bestMatchScore = -1;

          for (const [fName, data] of appState.modDataCache.entries()) {
            if (String(data.local_info?.mod_id) === String(item.modId)) {
              let score = 0;
              const minLen = Math.min(fName.length, mod.name.length);
              for (let i = 0; i < minLen; i++) {
                if (fName[i] === mod.name[i]) score++;
                else break;
              }
              if (score > bestMatchScore) {
                bestMatchScore = score;
                bestMatch = fName;
              }
            }
          }
          oldFolderName = bestMatch;
        }

        if (oldFolderName && oldFolderName !== mod.name) {
          const userChoice = await window.customConflictDialog(
            i18n.get('folderConflictMsg', { oldName: oldFolderName, newName: mod.name }),
            i18n.get('folderConflictTitle'),
            i18n.get('btnReplace'),
            i18n.get('btnKeepBoth'),
            i18n.get('cancelBtn')
          );

          if (userChoice === 'cancel') {
            logInfo(`User cancelled installation due to conflict: ${mod.name}`);

            try {
              await invoke('delete_mod', { modName: mod.name });
              if (appState.gamePath) {
                const settingsPath = await join(appState.gamePath, 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
                const content = await readTextFile(settingsPath);
                appState.xmlDoc = new DOMParser().parseFromString(content, 'application/xml');
              }
            } catch (err) {
              console.warn(`Failed to cleanup rejected mod folder ${mod.name}:`, err);
            }

            const currentItem = downloadHistory.find(d => d.id === item.id);
            if (currentItem) {
              currentItem.statusText = i18n.get('statusCancelled');
              currentItem.statusClass = 'cancelled';
              renderDownloadHistory();
              await saveDownloadHistory(downloadHistory);
            }

            setTimeout(() => {
              if (currentItem && currentItem.statusClass === 'cancelled') {
                currentItem.statusText = i18n.get('statusUnpacked');
                currentItem.statusClass = 'unpacked';
                renderDownloadHistory();
                saveDownloadHistory(downloadHistory);
              }
            }, 4000);

            return;
          }

          const shouldReplace = userChoice === 'replace';
          if (shouldReplace) {
            try {
              const updatedXmlContent = await invoke('update_mod_name_in_xml', {
                oldName: oldFolderName.toUpperCase(),
                newName: mod.name.toUpperCase(),
              });
              await loadXmlContent(updatedXmlContent, appState.currentFilePath);
              await invoke('delete_mod', { modName: oldFolderName });
              isRenamedEntry = true;
            } catch (e) {
              console.warn('Failed to process folder rename:', e);
            }
          }

          hasPromptedForRename = true;
        }

        if (!isRenamedEntry) {
          addNewModToXml(mod.name);
        }

        await invoke('ensure_mod_info', {
          modFolderName: mod.name,
          modId: item.modId || '',
          fileId: item.fileId || '',
          version: item.version || '',
          installSource: item.fileName,
        });

        await checkForAndLinkMod(mod.name);
        processedNames.add(mod.name);
      }

      item.modFolderName = analysis.successes[0].name;
    }

    await saveChanges();
    await renderModList();
    await renderDownloadHistory();

    updateDownloadItemStatus(item.id, i18n.get('statusInstalled'), 'installed');
    await saveDownloadHistory(downloadHistory);
    setDownloadHistory(downloadHistory);

    updateModDisplayState(item.modId);
    await saveCurrentProfile();
  }

  async function handleDownloadItemInstall(downloadId, isUpdate = false) {
    const downloadHistory = getDownloadHistory();
    const item = downloadHistory.find(d => d.id === downloadId);
    if (!item || !item.archivePath) {
      console.error('Attempted to install an item with no archive path:', item);
      return;
    }

    try {
      updateDownloadItemStatus(downloadId, isUpdate ? i18n.get('statusUpdating') : i18n.get('statusWaiting'), 'progress');

      const analysis = await invoke('install_mod_from_archive', {
        archivePathStr: item.archivePath,
        downloadId,
      });

      if (analysis.selection_needed) {
        const userResult = await openFolderSelectionModal(analysis.available_folders, item.fileName, analysis.temp_id);

        if (!userResult) {
          updateDownloadItemStatus(downloadId, i18n.get('statusCancelled'), 'cancelled');
          setTimeout(() => {
            const current = getDownloadHistory().find(d => d.id === downloadId);
            if (current && current.statusClass === 'cancelled') {
              current.statusText = i18n.get('statusDownloaded');
              current.statusClass = 'success';
              renderDownloadHistory();
            }
          }, 5000);
          return;
        }

        const finalAnalysis = await invoke('finalize_installation', {
          libraryId: analysis.temp_id,
          selectedFolders: userResult.selected,
          flattenPaths: userResult.flatten,
        });

        await processInstallAnalysis(finalAnalysis, item, isUpdate);
      } else {
        await processInstallAnalysis(analysis, item, isUpdate);
      }
    } catch (error) {
      logError(`Installation failed for ${item.fileName}: ${error}`);
      updateDownloadItemStatus(downloadId, `${i18n.get('installFailedTitle')}: ${error}`, 'error');
      await saveDownloadHistory(getDownloadHistory());
      await window.customAlert(`${i18n.get('installFailedTitle')}: ${error}`, 'Error');

      setTimeout(() => {
        const current = getDownloadHistory().find(d => d.id === downloadId);
        if (current && current.statusClass === 'error') {
          current.statusText = i18n.get('statusDownloaded');
          current.statusClass = 'success';
          renderDownloadHistory();
          saveDownloadHistory(getDownloadHistory());
        }
      }, 10000);
    }
  }

  async function startModDownload(
    { modId, fileId, version, fileName, displayName, replacingFileId, nxmQueryParams },
    isUpdate = false
  ) {
    logInfo(`Download Requested: ${displayName || fileName} (ID: ${modId}-${fileId}) [Update: ${isUpdate}]`);

    let downloadHistory = getDownloadHistory();
    const existingItem = downloadHistory.find(d => d.fileId === fileId);

    if (existingItem && existingItem.archivePath && !isUpdate) {
      const confirmed = await window.customConfirm(
        `You have already downloaded "${displayName || fileName}".\n\nDo you want to download it again and replace the existing file?`,
        'Duplicate Download'
      );

      if (!confirmed) {
        downloadHistoryModalOverlay.classList.remove('hidden');
        const next = downloadHistory.filter(d => d.fileId !== fileId);
        next.unshift(existingItem);
        setDownloadHistory(next);
        renderDownloadHistory();
        logInfo('Download skipped by user (Duplicate).');
        return;
      }

      downloadHistory = downloadHistory.filter(d => d.fileId !== fileId);
    }

    if (replacingFileId) {
      const oldVersionItem = downloadHistory.find(d => String(d.fileId) === String(replacingFileId));
      if (oldVersionItem && oldVersionItem.archivePath) {
        downloadHistory = downloadHistory.filter(d => d.id !== oldVersionItem.id);
      }
    }

    downloadHistoryModalOverlay.classList.remove('hidden');
    const downloadId = `download-${Date.now()}`;
    const newItemData = {
      id: downloadId,
      modId,
      fileId,
      version,
      displayName,
      fileName,
      statusText: isUpdate ? i18n.get('statusUpdating') : i18n.get('statusWaiting'),
      statusClass: 'progress',
      archivePath: null,
      modFolderName: null,
      size: 0,
      createdAt: 0,
    };

    downloadHistory.unshift(newItemData);
    setDownloadHistory(downloadHistory);
    renderDownloadHistory();

    try {
      updateDownloadItemStatus(downloadId, 'Requesting download URL...', 'progress');
      const downloadUrl = await nexusApi.fetchDownloadUrlFromNexus(modId, fileId, nxmQueryParams);
      if (!downloadUrl) {
        throw new Error('Could not retrieve download URL. (Check API Key or Premium Status)');
      }

      updateDownloadItemStatus(downloadId, i18n.get('statusDownloading'), 'progress');

      const downloadResult = await invoke('download_mod_archive', {
        downloadUrl,
        fileName,
        downloadId,
      });

      downloadHistory = getDownloadHistory();
      const item = downloadHistory.find(d => d.id === downloadId);
      if (!item) return;

      item.archivePath = downloadResult.path;
      item.size = downloadResult.size;
      item.createdAt = downloadResult.created_at;
      logInfo(`Download chain finished successfully for ${fileName}`);

      if (isUpdate) {
        await handleDownloadItemInstall(downloadId, true);
      } else {
        item.statusText = 'Downloaded';
        item.statusClass = 'success';
        await saveDownloadHistory(downloadHistory);
        renderDownloadHistory();
        if (localStorage.getItem('autoInstallAfterDownload') === 'true') {
          await handleDownloadItemInstall(downloadId);
        }
      }
    } catch (error) {
      console.error('Download/Update failed:', error);
      logError(`Frontend Download Error: ${error.message}`);
      updateDownloadItemStatus(downloadId, `Error: ${error.message}`, 'error');
      await saveDownloadHistory(getDownloadHistory());
    }
  }

  return {
    startModDownload,
    processInstallAnalysis,
    handleDownloadItemInstall,
  };
}
