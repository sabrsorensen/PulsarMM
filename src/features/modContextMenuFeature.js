export function createModContextMenuHandler(deps) {
  const {
    appState,
    modListContainer,
    modInfoPanel,
    priorityModalTitle,
    priorityModalDescription,
    priorityInput,
    priorityModalOverlay,
    i18n,
    invoke,
    join,
    readTextFile,
    loadXmlContent,
    saveDownloadHistory,
    saveCurrentProfile,
    renderModList,
    updateModDisplayState,
    getGamePath,
    getDownloadHistory,
    removeContextMenu,
    setContextMenu,
  } = deps;

  return async function handleModListContextMenu(e) {
    const modRow = e.target.closest('.mod-row');
    if (!modRow) return;

    e.preventDefault();
    e.stopPropagation();
    removeContextMenu();

    const clickedModName = modRow.dataset.modName;

    if (!appState.selectedModNames.has(clickedModName)) {
      appState.selectedModNames.clear();
      appState.selectedModNames.add(clickedModName);
      modListContainer.querySelectorAll('.mod-row.selected').forEach(el => el.classList.remove('selected'));
      modRow.classList.add('selected');
    }

    const contextMenu = document.createElement('div');
    contextMenu.className = 'context-menu';
    contextMenu.style.left = `${Math.min(e.clientX, window.innerWidth - 160)}px`;
    contextMenu.style.top = `${Math.min(e.clientY, window.innerHeight - 85)}px`;

    const selectionCount = appState.selectedModNames.size;

    if (selectionCount > 1) {
      const deleteButton = document.createElement('button');
      deleteButton.textContent = i18n.get('deleteModBtn', { modName: `${selectionCount} Mods` });
      deleteButton.className = 'context-menu-item delete';
      deleteButton.onclick = async () => {
        removeContextMenu();
        const confirmed = await window.customConfirm(
          `Are you sure you want to delete these ${selectionCount} mods?`,
          'Confirm Bulk Deletion'
        );

        if (confirmed) {
          let successCount = 0;
          const modsToDelete = Array.from(appState.selectedModNames);

          for (const modName of modsToDelete) {
            try {
              await invoke('delete_mod', { modName });

              const downloadHistory = getDownloadHistory();
              const deletedItem = downloadHistory.find(item => item.modFolderName && item.modFolderName.toUpperCase() === modName.toUpperCase());
              if (deletedItem) {
                deletedItem.statusText = i18n.get('statusUnpacked');
                deletedItem.statusClass = 'unpacked';
                deletedItem.modFolderName = null;
              }
              successCount++;
            } catch (err) {
              console.error(`Failed to delete ${modName}:`, err);
            }
          }

          const settingsPath = await join(getGamePath(), 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
          const content = await readTextFile(settingsPath);
          await loadXmlContent(content, settingsPath);
          await saveDownloadHistory(getDownloadHistory());
          await saveCurrentProfile();

          await window.customAlert(`Successfully deleted ${successCount} mods.`, 'Deleted');
        }
      };
      contextMenu.appendChild(deleteButton);
    } else {
      const modName = clickedModName;

      const renameButton = document.createElement('button');
      renameButton.textContent = i18n.get('renameBtn');
      renameButton.className = 'context-menu-item';
      renameButton.onclick = async () => {
        removeContextMenu();
        const newName = await window.customPrompt(
          `Enter new name for "${modName}":`,
          'Rename Mod',
          modName
        );

        if (newName && newName !== modName) {
          try {
            const newRenderList = await invoke('rename_mod_folder', {
              oldName: modName,
              newName,
            });

            const downloadHistory = getDownloadHistory();
            const historyItem = downloadHistory.find(item => item.modFolderName === modName);
            if (historyItem) {
              historyItem.modFolderName = newName;
              await saveDownloadHistory(downloadHistory);
            }

            await renderModList(newRenderList);
            const settingsPath = await join(getGamePath(), 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
            const content = await readTextFile(settingsPath);
            appState.xmlDoc = new DOMParser().parseFromString(content, 'application/xml');

            await saveCurrentProfile();
          } catch (err) {
            await window.customAlert(`Rename failed: ${err}`, 'Error');
          }
        }
      };
      contextMenu.appendChild(renameButton);

      const copyButton = document.createElement('button');
      copyButton.textContent = i18n.get('copyModNameBtn');
      copyButton.className = 'context-menu-item';
      copyButton.onclick = async () => {
        removeContextMenu();
        try {
          await navigator.clipboard.writeText(modName);
          await window.customAlert(i18n.get('copySuccess', { modName }), 'Success');
        } catch {
          // ignore clipboard failures
        }
      };
      contextMenu.appendChild(copyButton);

      const priorityButton = document.createElement('button');
      priorityButton.textContent = i18n.get('ctxChangePriority');
      priorityButton.className = 'context-menu-item';
      priorityButton.onclick = () => {
        removeContextMenu();
        const allRows = Array.from(modListContainer.querySelectorAll('.mod-row'));
        const modIndex = allRows.findIndex(row => row.dataset.modName === modName);
        const maxPriority = allRows.length - 1;
        priorityModalTitle.textContent = i18n.get('priorityModalTitleWithMod', { modName });
        priorityModalDescription.textContent = i18n.get('priorityModalDesc', { max: maxPriority });
        priorityInput.value = modIndex;
        priorityInput.max = maxPriority;
        priorityModalOverlay.dataset.modName = modName;
        priorityModalOverlay.classList.remove('hidden');
        setTimeout(() => priorityInput.focus(), 50);
      };
      contextMenu.appendChild(priorityButton);

      const deleteButton = document.createElement('button');
      deleteButton.textContent = i18n.get('deleteModBtn', { modName });
      deleteButton.className = 'context-menu-item delete';
      deleteButton.onclick = async () => {
        removeContextMenu();
        const confirmed = await window.customConfirm(
          i18n.get('confirmDeleteMod', { modName }),
          i18n.get('confirmDeleteTitle')
        );
        if (confirmed) {
          try {
            const modsToRender = await invoke('delete_mod', { modName });

            try {
              const settingsPath = await join(getGamePath(), 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
              const content = await readTextFile(settingsPath);
              appState.xmlDoc = new DOMParser().parseFromString(content, 'application/xml');
            } catch {
              location.reload();
              return;
            }

            await renderModList(modsToRender);

            const downloadHistory = getDownloadHistory();
            const deletedItem = downloadHistory.find(item => item.modFolderName && item.modFolderName.toUpperCase() === modName.toUpperCase());
            if (deletedItem) {
              deletedItem.statusText = i18n.get('statusUnpacked');
              deletedItem.statusClass = 'unpacked';
              deletedItem.modFolderName = null;
              await saveDownloadHistory(downloadHistory);
              if (deletedItem.modId) updateModDisplayState(deletedItem.modId);
            }

            await window.customAlert(i18n.get('deleteSuccess', { modName }), 'Deleted');
            await saveCurrentProfile();
          } catch (error) {
            await window.customAlert(`${i18n.get('deleteError', { modName })}\n\n${error}`, 'Error');
          }
        }
      };
      contextMenu.appendChild(deleteButton);
    }

    setContextMenu(contextMenu);
  };
}
