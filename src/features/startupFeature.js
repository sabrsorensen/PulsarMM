export function createInitializeApp(deps) {
  const {
    fetchCuratedData,
    i18n,
    loadDownloadHistory,
    invoke,
    validateLoginState,
    appState,
    rowPaddingSlider,
    rowPaddingValue,
    gridGapSlider,
    gridGapValue,
    modsPerPageSlider,
    modsPerPageValue,
    updateSliderFill,
    deckBExitsTextboxToggle,
    iconSteam,
    iconGog,
    iconXbox,
    openModsFolderBtn,
    settingsBtn,
    updateCheckBtn,
    enableAllBtn,
    disableAllBtn,
    dropZone,
    filePathLabel,
    join,
    readTextFile,
    loadXmlContent,
    listen,
    handleNxmLink,
    getDownloadHistory,
    renderModList,
    checkForUpdates,
    setCuratedDataPromise,
  } = deps;

  return async function initializeApp() {
    const savedLang = localStorage.getItem('selectedLanguage') || 'en';

    const curatedDataPromise = fetchCuratedData();
    setCuratedDataPromise(curatedDataPromise);

    const langPromise = i18n.loadLanguage(savedLang);
    const historyPromise = loadDownloadHistory();
    const migrationPromise = invoke('run_legacy_migration').catch(e => console.error('Migration error:', e));
    const gameDetectPromise = invoke('detect_game_installation');

    await Promise.all([langPromise, historyPromise, migrationPromise]);
    const loginPromise = validateLoginState();

    const savedPadding = localStorage.getItem('modRowPadding') || '5';
    document.documentElement.style.setProperty('--mod-row-vertical-padding', `${savedPadding}px`);
    rowPaddingSlider.value = savedPadding;
    rowPaddingValue.textContent = savedPadding;
    updateSliderFill(rowPaddingSlider);

    const savedGridGap = localStorage.getItem('browseGridGap') || '10';
    document.documentElement.style.setProperty('--browse-grid-gap', `${savedGridGap}px`);
    gridGapSlider.value = savedGridGap;
    gridGapValue.textContent = savedGridGap;
    updateSliderFill(gridGapSlider);

    const savedModsPerPage = localStorage.getItem('modsPerPage') || '20';
    appState.modsPerPage = parseInt(savedModsPerPage, 10);
    if (modsPerPageSlider) {
      modsPerPageSlider.value = appState.modsPerPage;
      modsPerPageValue.textContent = appState.modsPerPage;
      updateSliderFill(modsPerPageSlider);
    }

    const autoInstallToggle = document.getElementById('autoInstallToggle');
    autoInstallToggle.checked = localStorage.getItem('autoInstallAfterDownload') === 'true';
    autoInstallToggle.addEventListener('change', function () {
      localStorage.setItem('autoInstallAfterDownload', this.checked);
    });

    deckBExitsTextboxToggle.checked = localStorage.getItem('deckBExitsTextboxFirst') !== 'false';
    deckBExitsTextboxToggle.addEventListener('change', function () {
      localStorage.setItem('deckBExitsTextboxFirst', this.checked);
    });

    const gamePaths = await gameDetectPromise;
    if (gamePaths) {
      console.log(`Detected ${gamePaths.version_type} version of No Man's Sky.`);
      appState.gamePath = gamePaths.game_root_path;
      appState.settingsPath = gamePaths.settings_root_path;
      appState.versionType = gamePaths.version_type;
      const settingsInitialized = gamePaths.settings_initialized !== false;

      const launchBtn = document.getElementById('launchGameBtn');
      const launchIcon = document.getElementById('launchIcon');
      launchBtn.classList.remove('disabled');
      launchBtn.dataset.platform = appState.versionType;
      if (appState.versionType === 'Steam') launchIcon.src = iconSteam;
      else if (appState.versionType === 'GOG') launchIcon.src = iconGog;
      else if (appState.versionType === 'GamePass') launchIcon.src = iconXbox;

      if (!settingsInitialized) {
        const noticeKey = `settingsMissingNoticeShown:${appState.gamePath}`;
        console.warn('Game detected, but settings file is missing. Ask user to run No Man\'s Sky once, then reopen Pulsar.');
        if (localStorage.getItem(noticeKey) !== 'true' && typeof window.customAlert === 'function') {
          await window.customAlert(
            'No Man\'s Sky settings were not found yet. Launch the game once, then reopen Pulsar.',
            'Run Game First'
          );
          localStorage.setItem(noticeKey, 'true');
        }
      }
    }

    const hasGamePath = !!appState.gamePath;
    openModsFolderBtn.disabled = !hasGamePath;
    settingsBtn.classList.toggle('disabled', !hasGamePath);
    updateCheckBtn.classList.toggle('disabled', !hasGamePath);
    enableAllBtn.classList.toggle('disabled', !hasGamePath);
    disableAllBtn.classList.toggle('disabled', !hasGamePath);
    dropZone.classList.toggle('hidden', !hasGamePath);

    if (!hasGamePath) {
      const bannerText = document.querySelector('#globalBanner .banner-text');
      if (bannerText) bannerText.textContent = 'Game Not Found';
    }

    const hasSettingsFile = gamePaths && gamePaths.settings_initialized !== false;
    if (hasGamePath && appState.settingsPath && hasSettingsFile) {
      try {
        const settingsFilePath = await join(appState.settingsPath, 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
        const content = await readTextFile(settingsFilePath);
        if (content && content.length >= 10) {
          await loadXmlContent(content, settingsFilePath);
        }
      } catch (e) {
        console.warn('Could not auto-load settings file.', e);
      }
    }

    listen('nxm-link-received', (event) => handleNxmLink(event.payload));
    listen('install-progress', (event) => {
      const payload = event.payload;
      const item = getDownloadHistory().find(d => d.id === payload.id);
      if (item) {
        item.statusText = payload.step;
        const row = document.querySelector(`.download-item[data-download-id="${payload.id}"]`);
        if (row) {
          const statusEl = row.querySelector('.download-item-status');
          if (statusEl) statusEl.textContent = payload.step;
          const bar = row.querySelector('.download-progress-bar');
          if (bar && payload.progress !== undefined && payload.progress !== null) {
            bar.style.width = `${payload.progress}%`;
            bar.classList.remove('indeterminate');
          } else if (bar) {
            bar.style.width = '100%';
            bar.style.opacity = '0.5';
          }
        }
      }
    });

    if (appState.gamePath) {
      invoke('get_library_path').then(libPath => {
        const getDrive = (p) => {
          if (!p) return null;
          const m = p.match(/([a-zA-Z]):/);
          return m ? m[1].toUpperCase() : null;
        };

        const gameDrive = getDrive(appState.gamePath);
        const libDrive = getDrive(libPath);
        const suppressDriveCheck = localStorage.getItem('suppressDriveCheck') === 'true';

        if (gameDrive && libDrive && gameDrive !== libDrive && !suppressDriveCheck) {
          // Non-blocking startup: avoid modal prompts that can lock input if dialog rendering fails.
          console.warn(
            `Library/Game drive mismatch detected (Game: ${gameDrive}, Library: ${libDrive}). ` +
            'Open Settings to move the library path if desired.'
          );
        }
      }).catch(console.warn);
    }

    const suppressWarning = localStorage.getItem('suppressUntrackedWarning') === 'true';
    if (!suppressWarning) {
      invoke('check_for_untracked_mods').then(hasUntracked => {
        if (hasUntracked) {
          // Non-blocking startup warning; keep the app interactive.
          console.warn('Untracked mods detected. Use Troubleshoot/Warnings settings to review and suppress warning prompts.');
          renderModList();
        }
      }).catch(console.warn);
    }

    loginPromise.then(() => {
      i18n.updateUI();
    });

    curatedDataPromise.then(() => {
      if (appState.gamePath && appState.modDataCache.size > 0) {
        checkForUpdates(true);
      }
    });

    try {
      const pendingLink = await invoke('check_startup_intent');
      if (pendingLink) {
        console.log('Found pending startup NXM link:', pendingLink);
        handleNxmLink(pendingLink);
      }
    } catch (e) {
      console.error('Failed to check startup intent:', e);
    }
  };
}
