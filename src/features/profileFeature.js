export function createProfileFeature(deps) {
  const {
    invoke,
    appState,
    i18n,
    profileSelect,
    applyProfileBtn,
    mpProfileList,
    browseView,
    browseGridContainer,
    getDownloadHistory,
    saveDownloadHistory,
    renderDownloadHistory,
  } = deps;

  async function refreshProfileList() {
    try {
      const profiles = await invoke('list_profiles');
      const currentSelection = profileSelect.value;

      profileSelect.innerHTML = '';
      profiles.forEach(p => {
        const opt = document.createElement('option');
        opt.value = p;
        opt.textContent = p;
        profileSelect.appendChild(opt);
      });

      if (profiles.includes(currentSelection)) {
        profileSelect.value = currentSelection;
      } else {
        profileSelect.value = 'Default';
      }

      updateApplyButtonVisibility();
    } catch (err) {
      console.error('Failed to refresh profiles:', err);
    }
  }

  async function renderManagerList(selectedProfileInModalRef) {
    const profiles = await invoke('list_profiles');
    mpProfileList.innerHTML = '';

    profiles.forEach(p => {
      const li = document.createElement('li');
      li.className = 'mp-list-item';
      li.textContent = p;

      if (p === selectedProfileInModalRef.get()) li.classList.add('active');
      else if (!selectedProfileInModalRef.get() && p === appState.activeProfile) {
        li.classList.add('active');
        selectedProfileInModalRef.set(p);
      }

      li.onclick = () => {
        document.querySelectorAll('.mp-list-item').forEach(el => el.classList.remove('active'));
        li.classList.add('active');
        selectedProfileInModalRef.set(p);
      };

      li.ondblclick = () => {
        selectedProfileInModalRef.set(p);
        document.getElementById('mpSelectBtn').click();
      };

      mpProfileList.appendChild(li);
    });
  }

  function updateApplyButtonVisibility() {
    if (profileSelect.value !== appState.activeProfile) {
      applyProfileBtn.classList.remove('hidden');
    } else {
      applyProfileBtn.classList.add('hidden');
    }
  }

  function getDetailedInstalledMods() {
    const installedMods = [];
    const seen = new Set();

    getDownloadHistory().forEach(item => {
      if (item.statusClass === 'installed' && item.fileName) {
        if (!seen.has(item.fileName)) {
          seen.add(item.fileName);
          installedMods.push({
            filename: item.fileName,
            mod_id: item.modId ? String(item.modId) : null,
            file_id: item.fileId ? String(item.fileId) : null,
            version: item.version ? String(item.version) : null,
          });
        }
      }
    });
    return installedMods;
  }

  async function saveCurrentProfile() {
    if (appState.isPopulating) return;
    if (!appState.activeProfile) return;

    const modsData = getDetailedInstalledMods();

    try {
      await invoke('save_active_profile', {
        profileName: appState.activeProfile,
        mods: modsData,
      });
      console.log(`Auto-saved profile: ${appState.activeProfile}`);
    } catch (e) {
      console.error('Failed to auto-save profile:', e);
    }
  }

  async function syncDownloadHistoryWithProfile(profileName) {
    try {
      const profileFiles = await invoke('get_profile_mod_list', { profileName });

      const downloadHistory = getDownloadHistory();
      const allFilenames = downloadHistory.map(item => item.fileName).filter(n => n);
      const libraryMap = await invoke('check_library_existence', { filenames: allFilenames });

      let changed = false;

      downloadHistory.forEach(item => {
        const isInstalled = profileFiles.includes(item.fileName);
        const isUnpacked = libraryMap[item.fileName];

        let newStatusClass = '';
        let newStatusText = '';

        if (isInstalled) {
          newStatusClass = 'installed';
          newStatusText = i18n.get('statusInstalled');
        } else if (isUnpacked) {
          newStatusClass = 'unpacked';
          newStatusText = i18n.get('statusUnpacked');
        } else if (item.statusClass === 'installed' || item.statusClass === 'unpacked' || item.statusClass === 'success') {
          newStatusClass = 'success';
          newStatusText = i18n.get('statusDownloaded');
        }

        if (newStatusClass && item.statusClass !== newStatusClass) {
          if (item.statusClass !== 'error' && item.statusClass !== 'progress' && item.statusClass !== 'cancelled') {
            item.statusClass = newStatusClass;
            item.statusText = newStatusText;
            changed = true;
          }
          if (item.statusClass === 'installed' && !isInstalled) {
            item.statusClass = isUnpacked ? 'unpacked' : 'success';
            item.statusText = isUnpacked ? i18n.get('statusUnpacked') : i18n.get('statusDownloaded');
            changed = true;
          }
        }
      });

      if (changed) {
        await saveDownloadHistory(downloadHistory);
        console.log('Download history synchronized with profile.');
      }

      if (!document.getElementById('downloadHistoryModalOverlay').classList.contains('hidden')) {
        renderDownloadHistory();
      }

      if (!browseView.classList.contains('hidden')) {
        const allCards = browseGridContainer.querySelectorAll('.mod-card');
        allCards.forEach(() => {});
      }
    } catch (e) {
      console.error('Failed to sync download history:', e);
    }
  }

  return {
    refreshProfileList,
    renderManagerList,
    updateApplyButtonVisibility,
    getDetailedInstalledMods,
    saveCurrentProfile,
    syncDownloadHistoryWithProfile,
  };
}
