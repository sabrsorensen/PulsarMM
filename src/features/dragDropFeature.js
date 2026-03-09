export function createDragDropSetup(deps) {
  const {
    appWindow,
    dropZone,
    dragState,
    appState,
    basename,
    invoke,
    i18n,
    getDownloadHistory,
    setDownloadHistory,
    renderDownloadHistory,
    openFolderSelectionModal,
    processInstallAnalysis,
    saveDownloadHistory,
    join,
  } = deps;

  return async function setupDragAndDrop() {
    const showHighlight = () => dropZone.classList.add('drag-over');
    const hideHighlight = () => dropZone.classList.remove('drag-over');

    const onDragEnter = () => {
      if (dragState.draggedElement) return;
      showHighlight();
    };

    const onDragLeave = () => {
      hideHighlight();
    };

    const onDrop = async (event) => {
      if (dragState.draggedElement) return;
      hideHighlight();

      let files = event.payload;
      if (files && files.paths) files = files.paths;
      if (!files || !Array.isArray(files) || files.length === 0) return;

      window.addAppLog(`File Drop Detected: ${files.length} paths received.`, 'INFO');
      if (!appState.xmlDoc) {
        await window.customAlert('Please load a GCMODSETTINGS.MXML file first.', 'Error');
        return;
      }

      const archiveFiles = files.filter(p => /\.(zip|rar|7z)$/i.test(p));
      if (archiveFiles.length === 0) {
        window.addAppLog('File Drop ignored: No valid archives found in drop.', 'WARN');
        return;
      }

      for (const filePath of archiveFiles) {
        const fileName = await basename(filePath);
        const downloadHistory = getDownloadHistory();
        const existingIndex = downloadHistory.findIndex(d => d.fileName === fileName);
        if (existingIndex > -1) downloadHistory.splice(existingIndex, 1);

        window.addAppLog(`Processing dropped file: ${fileName}`, 'INFO');

        const downloadId = `manual-${Date.now()}`;
        const newItem = {
          id: downloadId,
          modId: '',
          fileId: '',
          version: 'Manual',
          displayName: fileName,
          fileName,
          statusText: i18n.get('statusWaiting') || 'Installing...',
          statusClass: 'progress',
          archivePath: null,
          modFolderName: null,
          size: 0,
          createdAt: Date.now() / 1000,
        };

        downloadHistory.unshift(newItem);
        setDownloadHistory(downloadHistory);
        renderDownloadHistory();

        try {
          const analysis = await invoke('install_mod_from_archive', {
            archivePathStr: filePath,
            downloadId,
          });

          if (analysis.active_archive_path) {
            newItem.archivePath = analysis.active_archive_path;
          }

          let finalResult = analysis;
          if (analysis.selection_needed) {
            const userResult = await openFolderSelectionModal(analysis.available_folders, fileName, analysis.temp_id);

            if (!userResult) {
              await invoke('clean_staging_folder');
              newItem.statusText = i18n.get('statusCancelled') || 'Cancelled';
              newItem.statusClass = 'cancelled';
              window.addAppLog(`User cancelled folder selection for: ${fileName}`, 'INFO');
              await saveDownloadHistory(downloadHistory);
              renderDownloadHistory();

              setTimeout(() => {
                const current = getDownloadHistory().find(d => d.id === downloadId);
                if (current && current.statusClass === 'cancelled') {
                  current.statusText = i18n.get('statusDownloaded');
                  current.statusClass = 'success';
                  renderDownloadHistory();
                  saveDownloadHistory(getDownloadHistory());
                }
              }, 5000);
              continue;
            }

            finalResult = await invoke('finalize_installation', {
              libraryId: analysis.temp_id,
              selectedFolders: userResult.selected,
              flattenPaths: userResult.flatten,
            });

            await processInstallAnalysis(finalResult, newItem, false);
          } else {
            await processInstallAnalysis(analysis, newItem, false);
          }

          const installedCount = finalResult.successes ? finalResult.successes.length : 0;
          const suppressSuccess = localStorage.getItem('suppressInstallSuccess') === 'true';

          if (!suppressSuccess && installedCount > 0) {
            const keepShowing = await window.customConfirm(
              i18n.get('installCompleteMsg', { count: installedCount, fileName }),
              i18n.get('installCompleteTitle'),
              i18n.get('okBtn'),
              i18n.get('dontShowAgainBtn')
            );
            if (keepShowing === false) localStorage.setItem('suppressInstallSuccess', 'true');
          }

          window.addAppLog(`Successfully installed dropped file: ${fileName}`, 'INFO');
        } catch (error) {
          window.addAppLog(`Drag/Drop install failed for ${fileName}: ${error}`, 'ERROR');
          console.error(`Error installing ${fileName}:`, error);

          newItem.statusText = 'Error';
          newItem.statusClass = 'error';
          renderDownloadHistory();
          await window.customAlert(`${i18n.get('installFailedTitle')}: ${error}`, 'Error');

          try {
            const downloadsDir = await invoke('get_downloads_path');
            const targetPath = await join(downloadsDir, fileName);
            await invoke('delete_archive_file', { path: targetPath });

            const next = getDownloadHistory().filter(d => d.id !== downloadId);
            setDownloadHistory(next);
            renderDownloadHistory();
            await saveDownloadHistory(next);
          } catch (delErr) {
            console.warn('Failed to cleanup bad zip:', delErr);
          }
        }
      }
    };

    await appWindow.listen('tauri://file-drop-hover', onDragEnter);
    await appWindow.listen('tauri://drag-enter', onDragEnter);
    await appWindow.listen('tauri://file-drop-cancelled', onDragLeave);
    await appWindow.listen('tauri://drag-leave', onDragLeave);
    await appWindow.listen('tauri://file-drop', onDrop);
    await appWindow.listen('tauri://drag-drop', onDrop);
  };
}
