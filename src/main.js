import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile, mkdir } from "@tauri-apps/plugin-fs";
import { basename, join, appDataDir } from "@tauri-apps/api/path";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

// --- IMPORT LOCALES ---
import localeEn from '../src-tauri/locales/en.json';
import localePt from '../src-tauri/locales/pt.json';

const bundledLocales = {
  en: localeEn,
  pt: localePt,
};

// --- IMPORT ASSETS ---
import iconSteam from './assets/icon-steam.png';
import iconGog from './assets/icon-gog.png';
import iconXbox from './assets/icon-xbox.png';
import iconNexus from './assets/icon-nexus.png';
import iconMaximize from './assets/icon-maximize.png';
import iconRestore from './assets/icon-restore.png';

// --- IMPORT GAMEPAD ---
import { GamepadManager } from './gamepad.js';
import {
  DEFAULT_WIDTH,
  PANEL_OPEN_WIDTH,
  SCROLL_SPEED,
  createAppState,
  createDragState,
  createScrollState,
  createDownloadSortState,
} from './state/uiState.js';
import { loadImageViaTauri } from './utils/imageUtils.js';
import { formatBytes, formatDate, formatNexusDate, mapLangCode } from './utils/formatUtils.js';
import { getBaseName, isNewerVersionAvailable } from './utils/versionUtils.js';
import { bbcodeToHtml } from './utils/textUtils.js';
import { formatNode } from './utils/xmlUtils.js';
import { createBrowseFeature } from './features/browseFeature.js';
import { createCuratedDataFeature } from './features/curatedDataFeature.js';
import { createDownloadContextMenuFeature } from './features/downloadContextMenuFeature.js';
import { createDownloadHistoryFeature } from './features/downloadHistoryFeature.js';
import { createModContextMenuHandler } from './features/modContextMenuFeature.js';
import { createNexusAuthFeature } from './features/nexusAuthFeature.js';
import { createNxmHandlerFeature } from './features/nxmHandlerFeature.js';
import { createProfileFeature } from './features/profileFeature.js';
import { createSettingsHelpers } from './features/settingsFeature.js';
import { installDialogHelpers } from './features/dialogFeature.js';
import { createLaunchGameHandler } from './features/launchGameFeature.js';
import { createNexusApi } from './features/nexusApi.js';
import { createInitializeApp } from './features/startupFeature.js';
import { createUpdateFeature } from './features/updateFeature.js';
import { createFolderSelectionModal } from './features/folderSelectionFeature.js';
import { createDragDropSetup } from './features/dragDropFeature.js';
import { createDownloadInstallFeature } from './features/downloadInstallFeature.js';
import { installGlobalHotkeys } from './features/hotkeyFeature.js';
import { resolveUpdateCandidate } from './features/updateResolver.js';

// Get the window instance for listener attachment
const appWindow = getCurrentWindow();

// --- Global State & Constants ---
let NEXUS_API_KEY = "";
let curatedData = [];
let curatedDataPromise = null;
let downloadHistory = [];
const nexusFileCache = new Map();

let isPanelOpen = false;

// Image loader moved to utils/imageUtils.js

document.addEventListener('DOMContentLoaded', () => {
  try {

  // --- Application & UI State ---
  const appState = createAppState();

  const dragState = createDragState();

  const scrollState = createScrollState();

  let contextMenu = null;
  let gamepad = null;
  let updateNXMButtonStateRef = null;

  const downloadSortState = createDownloadSortState();

  const nexusApi = createNexusApi({
    invoke,
    getApiKey: () => NEXUS_API_KEY,
    cache: nexusFileCache,
  });

  // --- UI Element References ---
  const loadFileBtn = document.getElementById('loadFileBtn'),
    openModsFolderBtn = document.getElementById('openModsFolderBtn'),
    filePathLabel = document.getElementById('filePathLabel'),
    disableAllSwitch = document.getElementById('disableAllSwitch'),
    modListContainer = document.getElementById('modListContainer'),
    settingsBtn = document.getElementById('settingsBtn'),
    settingsModalOverlay = document.getElementById('settingsModalOverlay'),
    closeSettingsModalBtn = document.getElementById('closeSettingsModalBtn'),
    rowPaddingSlider = document.getElementById('rowPaddingSlider'),
    rowPaddingValue = document.getElementById('rowPaddingValue'),
    deleteSettingsBtn = document.getElementById('deleteSettingsBtn'),
    dropZone = document.getElementById('dropZone'),
    searchModsInput = document.getElementById('searchModsInput'),
    languageSelector = document.getElementById('languageSelector'),
    enableAllBtn = document.getElementById('enableAllBtn'),
    disableAllBtn = document.getElementById('disableAllBtn'),
    customCloseBtn = document.getElementById('customCloseBtn'),
    modInfoPanel = document.getElementById('modInfoPanel'),
    infoModName = document.getElementById('infoModName'),
    infoAuthor = document.getElementById('infoAuthor'),
    infoInstalledVersion = document.getElementById('infoInstalledVersion'),
    infoLatestVersion = document.getElementById('infoLatestVersion'),
    infoDescription = document.getElementById('infoDescription'),
    infoUpdateBtn = document.getElementById('infoUpdateBtn'),
    infoNexusLink = document.getElementById('infoNexusLink'),
    updateCheckBtn = document.getElementById('updateCheckBtn'),
    updateModalOverlay = document.getElementById('updateModalOverlay'),
    updateListContainer = document.getElementById('updateListContainer'),
    closeUpdateModalBtn = document.getElementById('closeUpdateModalBtn'),
    navMyMods = document.getElementById('navMyMods'),
    navBrowse = document.getElementById('navBrowse'),
    myModsView = document.getElementById('myModsView'),
    browseView = document.getElementById('browseView'),
    browseGridContainer = document.getElementById('browseGridContainer'),
    fileSelectionModalOverlay = document.getElementById('fileSelectionModalOverlay'),
    fileSelectionModalTitle = document.getElementById('fileSelectionModalTitle'),
    fileSelectionListContainer = document.getElementById('fileSelectionListContainer'),
    closeFileSelectionModalBtn = document.getElementById('closeFileSelectionModalBtn'),
    browseSearchInput = document.getElementById('browseSearchInput'),
    browseSortSelect = document.getElementById('browseSortSelect'),
    browseFilterSelect = document.getElementById('browseFilterSelect'),
    paginationContainer = document.getElementById('paginationContainer'),
    modDetailPanel = document.getElementById('modDetailPanel'),
    modDetailCloseBtn = document.getElementById('modDetailCloseBtn'),
    modDetailName = document.getElementById('modDetailName'),
    modDetailAuthor = document.getElementById('modDetailAuthor'),
    modDetailImage = document.getElementById('modDetailImage'),
    modDetailVersion = document.getElementById('modDetailVersion'),
    modDetailUpdated = document.getElementById('modDetailUpdated'),
    modDetailDescription = document.getElementById('modDetailDescription'),
    modDetailInstallBtnContainer = document.getElementById('modDetailInstallBtnContainer'),
    modDetailSecondaryActions = document.getElementById('modDetailSecondaryActions'),
    modDetailInstalled = document.getElementById('modDetailInstalled'),
    modDetailCreated = document.getElementById('modDetailCreated'),
    changelogModalOverlay = document.getElementById('changelogModalOverlay'),
    changelogModalTitle = document.getElementById('changelogModalTitle'),
    changelogListContainer = document.getElementById('changelogListContainer'),
    closeChangelogModalBtn = document.getElementById('closeChangelogModalBtn'),
    priorityModalOverlay = document.getElementById('priorityModalOverlay'),
    priorityModalTitle = document.getElementById('priorityModalTitle'),
    priorityModalDescription = document.getElementById('priorityModalDescription'),
    priorityInput = document.getElementById('priorityInput'),
    confirmPriorityBtn = document.getElementById('confirmPriorityBtn'),
    cancelPriorityBtn = document.getElementById('cancelPriorityBtn'),
    downloadHistoryBtn = document.getElementById('downloadHistoryBtn'),
    downloadHistoryModalOverlay = document.getElementById('downloadHistoryModalOverlay'),
    downloadListContainer = document.getElementById('downloadListContainer'),
    closeDownloadHistoryBtn = document.getElementById('closeDownloadHistoryBtn'),
    clearDownloadHistoryBtn = document.getElementById('clearDownloadHistoryBtn'),
    nxmHandlerBtn = document.getElementById('nxmHandlerBtn'),
    gridGapSlider = document.getElementById('gridGapSlider'),
    gridGapValue = document.getElementById('gridGapValue'),
    deckBExitsTextboxToggle = document.getElementById('deckBExitsTextboxToggle'),
    modsPerPageSlider = document.getElementById('modsPerPageSlider'),
    modsPerPageValue = document.getElementById('modsPerPageValue');
  const currentGamePathEl = document.getElementById('currentGamePath');

  // --- Core Application Logic ---

  // --- LOGGING SYSTEM ---
  window.addAppLog = async (message, level = 'INFO') => {
    try {
      // Print to DevTools
      if (level === 'ERROR') console.error(message);
      else console.log(message);

      // Send to Rust to write to disk
      await invoke('write_to_log', { level, message: String(message) });
    } catch (e) {
      console.error("Failed to write log:", e);
    }
  };

  // Log startup
  window.addAppLog("Pulsar Mod Manager Started", "INFO");

  // --- GLOBAL HOTKEYS & INPUT HANDLING ---
  installGlobalHotkeys({
    getInputMode: () => gamepad?.inputMode,
    appState,
    modListContainer,
    modInfoPanel,
    downloadListContainer,
  });

  // --- AUTO-REFRESH ON FOCUS ---
  // If the user deletes a mod in Explorer and tabs back, this updates everything.
  window.addEventListener('focus', async () => {
    // Only run if we are fully initialized and on the "My Mods" view
    if (appState.activeProfile && !appState.isPopulating && !myModsView.classList.contains('hidden')) {

      console.log("App focused. Syncing with disk...");

      try {
        // 1. Call Rust: This cleans the file on disk and returns the correct list
        const cleanList = await invoke('get_all_mods_for_render');

        // 2. Reload the in-memory XML from the disk
        if (appState.currentFilePath) {
          const freshContent = await readTextFile(appState.currentFilePath);
          appState.xmlDoc = new DOMParser().parseFromString(freshContent, "application/xml");
        }

        // 3. Update the UI with the clean list
        await renderModList(cleanList);

        // 4. Update Profile JSON to match the new reality
        await saveCurrentProfile();

        // 5. Sync Download History visuals if the modal happens to be open
        if (!downloadHistoryModalOverlay.classList.contains('hidden')) {
          await syncDownloadHistoryWithProfile(appState.activeProfile);
          renderDownloadHistory();
        }
      } catch (e) {
        console.warn("Auto-refresh failed:", e);
      }
    }
  });

  // --- Monitor Window Resizing ---
  // This debounces the event so it only logs once when the resizing STOPS.
  let resizeLogTimeout;
  window.addEventListener('resize', () => {
    clearTimeout(resizeLogTimeout);
    resizeLogTimeout = setTimeout(async () => {
      try {
        const size = await appWindow.innerSize();
        const pos = await appWindow.outerPosition();
        window.addAppLog(`Window Resized to: ${size.width}x${size.height} at (${pos.x}, ${pos.y})`, "INFO");
      } catch (e) { /* ignore */ }
    }, 500);
  });

  installDialogHelpers({
    getText: (key) => appState.currentTranslations?.[key] || key,
  });

  const i18n = {
    async loadLanguage(lang) {
      try {
        // 1. Load English Base (bundled in JS)
        const enData = bundledLocales.en;

        if (lang === 'en' || !bundledLocales[lang]) {
          appState.currentTranslations = enData;
        } else {
          // 2. Merge Target over English
          appState.currentTranslations = { ...enData, ...bundledLocales[lang] };
        }

        localStorage.setItem('selectedLanguage', lang);

        // 3. Refresh UI
        this.updateUI();

      } catch (e) {
        console.error(`Failed to load language for ${lang}`, e);
        appState.currentTranslations = bundledLocales.en;
      }
    },
    updateUI() {
      // 1. Sync Dropdown Value (FIXES THE DROPDOWN RESET BUG)
      const currentLang = localStorage.getItem('selectedLanguage') || 'en';
      if (languageSelector) {
        languageSelector.value = currentLang;
      }

      // 2. Auto-translate static elements
      document.querySelectorAll('[data-i18n]').forEach(el => {
        // SKIP the NXM button here to prevent it from flickering/resetting incorrectly
        if (el.id === 'nxmHandlerBtn') return;

        const key = el.getAttribute('data-i18n');
        const attributeName = el.getAttribute('data-i18n-attr');
        if (appState.currentTranslations[key]) {
          const translatedText = appState.currentTranslations[key];
          if (attributeName) {
            el.setAttribute(attributeName, translatedText);
          } else {
            el.textContent = translatedText;
          }
        }
      });

      // 3. Handle Nexus Login Status
      const nexusStatus = document.getElementById('nexusAccountStatus');
      const nexusBtn = document.getElementById('nexusAuthBtn');

      if (appState.nexusUsername) {
        if (nexusStatus) nexusStatus.textContent = this.get('statusConnectedAs', { name: appState.nexusUsername });
        if (nexusBtn) nexusBtn.textContent = this.get('disconnectBtn');
      } else {
        if (nexusStatus) nexusStatus.textContent = this.get('statusNotLoggedIn');
        if (nexusBtn) nexusBtn.textContent = this.get('connectBtn');
      }

      // 4. Handle NXM Button State (FIXES THE BUTTON RESET BUG)
      // We explicitly call this logic *after* translations are applied
      // to ensure the button text reflects the actual logic (Registered vs Not Registered)
      if (typeof updateNXMButtonStateRef === 'function') {
        updateNXMButtonStateRef();
      }

      // 5. Update File Path Label
      if (appState.currentFilePath) {
        basename(appState.currentFilePath).then(fileNameWithExt => {
          const fileNameWithoutExt = fileNameWithExt.slice(0, fileNameWithExt.lastIndexOf('.'));
          filePathLabel.textContent = this.get('editingFile', { fileName: fileNameWithoutExt });
        });
      } else {
        filePathLabel.textContent = this.get('noFileLoaded');
      }

      this.adjustBannerWidths();
    },
    get(key, placeholders = {}) {
      let text = appState.currentTranslations[key] || key;
      for (const [placeholder, value] of Object.entries(placeholders)) {
        text = text.replace(`{{${placeholder}}}`, value);
      }
      return text;
    },
    adjustBannerWidths() {
      requestAnimationFrame(() => {
        const HORIZONTAL_PADDING = 60;
        const bannerConfigs = [
          { id: 'globalBanner', minWidth: 240 },
          { id: 'individualBanner', minWidth: 310 }
        ];
        bannerConfigs.forEach(config => {
          const banner = document.getElementById(config.id);
          if (banner) {
            const textElement = banner.querySelector('.banner-text');
            if (textElement) {
              const calculatedWidth = textElement.scrollWidth + HORIZONTAL_PADDING;
              const finalWidth = Math.max(config.minWidth, calculatedWidth);
              banner.style.width = `${finalWidth}px`;
            }
          }
        });
      });
    }
  };

  const browseFeature = createBrowseFeature({
    appState,
    getCuratedData: () => curatedData,
    i18n,
    invoke,
    loadImageViaTauri,
    mapLangCode,
    formatNexusDate,
    bbcodeToHtml,
    getBaseName,
    isNewerVersionAvailable,
    appWindow,
    LogicalSize,
    PANEL_OPEN_WIDTH,
    getIsPanelOpen: () => isPanelOpen,
    setIsPanelOpen: (next) => { isPanelOpen = next; },
    elements: {
      browseGridContainer,
      browseSearchInput,
      browseSortSelect,
      browseFilterSelect,
      languageSelector,
      modDetailPanel,
      modDetailName,
      modDetailAuthor,
      modDetailImage,
      modDetailVersion,
      modDetailUpdated,
      modDetailDescription,
      modDetailInstallBtnContainer,
      modDetailSecondaryActions,
      modDetailInstalled,
      modDetailCreated,
      fileSelectionModalOverlay,
      fileSelectionModalTitle,
      fileSelectionListContainer,
      changelogModalOverlay,
      changelogModalTitle,
      changelogListContainer,
      paginationContainer,
    },
  });

  const {
    fetchAndRenderBrowseGrid,
    filterAndDisplayMods,
    openModDetailPanel,
    showFileSelectionModal,
  } = browseFeature;

  let downloadInstallFeature = null;

  const downloadHistoryFeature = createDownloadHistoryFeature({
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
    getCuratedData: () => curatedData,
    getDownloadHistory: () => downloadHistory,
    setDownloadHistory: (next) => { downloadHistory = next; },
    onInstall: (downloadId) => downloadInstallFeature?.handleDownloadItemInstall(downloadId),
    onContextMenu: (e, downloadId) => downloadContextMenuFeature.showDownloadContextMenu(e, downloadId),
    onStartModDownload: (payload) => downloadInstallFeature?.startModDownload(payload),
    elements: {
      downloadListContainer,
      browseGridContainer,
      modDetailPanel,
      modDetailName,
      modDetailInstallBtnContainer,
      modDetailInstalled,
    },
  });

  const {
    loadDownloadHistory,
    saveDownloadHistory,
    renderDownloadHistory,
    updateDownloadStatus,
    handleNxmLink,
    updateModDisplayState,
  } = downloadHistoryFeature;

  const downloadContextMenuFeature = createDownloadContextMenuFeature({
    appState,
    downloadListContainer,
    i18n,
    invoke,
    getDownloadHistory: () => downloadHistory,
    renderDownloadHistory,
    saveDownloadHistory,
    handleDownloadItemInstall: (downloadId) => downloadInstallFeature?.handleDownloadItemInstall(downloadId),
    removeContextMenu: (...args) => removeContextMenu(...args),
    setContextMenu: (...args) => setContextMenu(...args),
  });

  const curatedDataFeature = createCuratedDataFeature({
    appDataDir,
    join,
    readTextFile,
    writeTextFile,
    mkdir,
    invoke,
    setCuratedData: (next) => { curatedData = next; },
  });

  const { fetchCuratedData } = curatedDataFeature;

  // --- NEXUS LOGIN LOGIC ---
  const nexusAuthBtn = document.getElementById('nexusAuthBtn');
  const nexusAccountStatus = document.getElementById('nexusAccountStatus');
  const nexusAuthFeature = createNexusAuthFeature({
    invoke,
    i18n,
    appState,
    nexusAuthBtn,
    nexusAccountStatus,
    setApiKey: (next) => { NEXUS_API_KEY = next; },
  });
  const { validateLoginState, handleAuthButtonClick } = nexusAuthFeature;

  const nxmHandlerFeature = createNxmHandlerFeature({
    nxmHandlerBtn,
    i18n,
    invoke,
    updateNXMButtonState: (...args) => updateNXMButtonState(...args),
  });
  const { handleNxmHandlerClick } = nxmHandlerFeature;

  const initializeApp = createInitializeApp({
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
    loadXmlContent: async (...args) => loadXmlContent(...args),
    listen,
    handleNxmLink,
    getDownloadHistory: () => downloadHistory,
    renderModList: async (...args) => renderModList(...args),
    checkForUpdates: (...args) => checkForUpdates(...args),
    setCuratedDataPromise: (promise) => { curatedDataPromise = promise; },
  });

  const loadXmlContent = async (content, path) => {
    try {
      appState.currentFilePath = path;
      const fileNameWithExt = await basename(appState.currentFilePath);
      const fileNameWithoutExt = fileNameWithExt.slice(0, fileNameWithExt.lastIndexOf('.'));
      filePathLabel.textContent = i18n.get('editingFile', { fileName: fileNameWithoutExt });
      appState.xmlDoc = new DOMParser().parseFromString(content, "application/xml");
      await renderModList();
    } catch (e) {
      console.error('[loadXmlContent] Error:', e);
    }
  };

  const renderModList = async (directData = null) => {
    if (!directData && !appState.xmlDoc) return;
    const scrollPos = modListContainer.scrollTop;
    appState.isPopulating = true;
    modListContainer.innerHTML = '';
    appState.installedModsMap.clear();
    const suppressUntracked = localStorage.getItem('suppressUntrackedWarning') === 'true';
    const disableAllNode = appState.xmlDoc.querySelector('Property[name="DisableAllMods"]');
    if (disableAllNode) {
      disableAllSwitch.checked = disableAllNode.getAttribute('value').toLowerCase() === 'true';
      disableAllSwitch.disabled = false;
    }
    let modsToRender;
    if (directData) {
      modsToRender = directData;
    } else {
      modsToRender = await invoke('get_all_mods_for_render');
    }

    appState.modDataCache.clear();
    modsToRender.forEach(modData => {
      appState.modDataCache.set(modData.folder_name, modData);
    });

    modsToRender.forEach((modData, index) => {
      if (modData.local_info) {
        const { mod_id, file_id, version } = modData.local_info;
        if (mod_id && file_id && version) {
          const modIdStr = String(mod_id);
          if (!appState.installedModsMap.has(modIdStr)) {
            appState.installedModsMap.set(modIdStr, new Map());
          }
          appState.installedModsMap.get(modIdStr).set(String(file_id), version);
        }
      }

      const row = document.createElement('div');
      row.className = 'mod-row';
      row.dataset.modName = modData.folder_name;
      const showRedDot = !suppressUntracked && (!modData.local_info || !modData.local_info.install_source);

      const untrackedHtml = showRedDot
        ? `<span class="untracked-indicator" title="${i18n.get('untrackedModTooltip')}"></span>`
        : '';

      row.innerHTML = `
                <div class="mod-name-container">
                    <span class="mod-name-text">${modData.folder_name}</span>
                    ${untrackedHtml}
                    <span class="update-indicator hidden" data-i18n-title="updateAvailableTooltip" title="Update available"></span>
                </div>
                <div class="priority"><input type="text" class="priority-input" value="${index}" readonly></div>
                <div class="enabled"><label class="switch"><input type="checkbox" class="enabled-switch" ${modData.enabled ? 'checked' : ''}><span class="slider"></span></label></div>
            `;

      row.querySelector('.enabled-switch').addEventListener('change', async (e) => {
        const newState = e.target.checked;
        window.addAppLog(`User toggled mod '${modData.folder_name}': ${newState ? 'ENABLED' : 'DISABLED'}`, "INFO");

        const modNode = Array.from(appState.xmlDoc.querySelectorAll('Property[name="Data"] > Property'))
          .find(node => {
            const nameProp = node.querySelector('Property[name="Name"]');
            return nameProp && nameProp.getAttribute('value').toUpperCase() === modData.folder_name.toUpperCase();
          });
        if (modNode) {
          const newVal = newState ? 'true' : 'false';
          const eNode = modNode.querySelector('Property[name="Enabled"]');
          const evrNode = modNode.querySelector('Property[name="EnabledVR"]');
          if (eNode) eNode.setAttribute('value', newVal);
          if (evrNode) evrNode.setAttribute('value', newVal);
          await saveChanges();
          await saveCurrentProfile();
        }
      });

      modListContainer.appendChild(row);
    });

    appState.isPopulating = false;
    filterModList();
    modListContainer.scrollTop = scrollPos;
  };

  function updateModListStates() {
    if (!appState.xmlDoc) return;

    const modRows = modListContainer.querySelectorAll('.mod-row');

    modRows.forEach(row => {
      const modName = row.dataset.modName;
      const modNode = Array.from(appState.xmlDoc.querySelectorAll('Property[name="Data"] > Property'))
        .find(node => {
          const nameProp = node.querySelector('Property[name="Name"]');
          return nameProp && nameProp.getAttribute('value').toUpperCase() === modName.toUpperCase();
        });

      if (modNode) {
        const isEnabled = modNode.querySelector('Property[name="Enabled"]')?.getAttribute('value').toLowerCase() === 'true';

        const checkbox = row.querySelector('.enabled-switch');
        if (checkbox && checkbox.checked !== isEnabled) {
          checkbox.checked = isEnabled;
        }
      }
    });
  }

  // --- APP AUTO-UPDATER LOGIC ---
  const updateFeature = createUpdateFeature({
    invoke,
    appState,
    getCuratedData: () => curatedData,
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
  });

  const { checkAppUpdate, checkForUpdates } = updateFeature;

  // --- Other Helper Functions ---
  function refreshBrowseTabBadges() {
    // Safety check
    if (!browseGridContainer || browseGridContainer.childElementCount === 0) return;

    const cards = browseGridContainer.querySelectorAll('.mod-card');
    cards.forEach(card => {
      const modId = card.dataset.modId;
      const isInstalled = appState.installedModsMap.has(String(modId));

      card.classList.toggle('is-installed', isInstalled);

      const badge = card.querySelector('.mod-card-installed-badge');
      if (badge) {
        badge.classList.toggle('hidden', !isInstalled);
      }
    });
  }

  function updateSliderFill(slider) {
    const val = slider.value;
    const min = slider.min || 0;
    const max = slider.max || 100;

    // Calculate percentage
    const percentage = ((val - min) / (max - min)) * 100;

    // Update background: Accent Color on Left | White on Right
    slider.style.background = `linear-gradient(to right, var(--c-accent-primary) ${percentage}%, #EAEAEA ${percentage}%)`;
  }

  // formatBytes/formatDate/getBaseName moved to utils modules

  // resolveUpdateCandidate moved to features/updateResolver.js

  function startModDownload(payload, isUpdate = false) {
    return downloadInstallFeature.startModDownload(payload, isUpdate);
  }

  function processInstallAnalysis(analysis, item, isUpdate) {
    return downloadInstallFeature.processInstallAnalysis(analysis, item, isUpdate);
  }

  function handleDownloadItemInstall(downloadId, isUpdate = false) {
    return downloadInstallFeature.handleDownloadItemInstall(downloadId, isUpdate);
  }

  // --- DOWNLOAD PATH SETTINGS ---
  const changeDownloadDirBtn = document.getElementById('changeDownloadDirBtn');
  const currentDownloadPathEl = document.getElementById('currentDownloadPath');
  const currentLibraryPathEl = document.getElementById('currentLibraryPath');
  const {
    updateDownloadPathUI,
    updateGamePathUI,
    updateLibraryPathUI,
    updateNXMButtonState,
  } = createSettingsHelpers({
    invoke,
    i18n,
    currentDownloadPathEl,
    currentGamePathEl,
    currentLibraryPathEl,
  });

  function applyDetectedGamePaths(gamePaths) {
    if (!gamePaths) return;
    appState.gamePath = gamePaths.game_root_path;
    appState.settingsPath = gamePaths.settings_root_path;
    appState.versionType = gamePaths.version_type;
    const launchBtn = document.getElementById('launchGameBtn');
    const launchIcon = document.getElementById('launchIcon');
    launchBtn.classList.remove('disabled');
    launchBtn.dataset.platform = appState.versionType;
    if (appState.versionType === 'Steam') launchIcon.src = iconSteam;
    else if (appState.versionType === 'GOG') launchIcon.src = iconGog;
    else if (appState.versionType === 'GamePass') launchIcon.src = iconXbox;
    openModsFolderBtn.disabled = false;
    settingsBtn.classList.remove('disabled');
    updateCheckBtn.classList.remove('disabled');
    enableAllBtn.classList.remove('disabled');
    disableAllBtn.classList.remove('disabled');
    dropZone.classList.remove('hidden');
    updateGamePathUI(appState.gamePath);
  }
  updateNXMButtonStateRef = updateNXMButtonState;

  changeDownloadDirBtn.addEventListener('click', async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select New Downloads Folder"
      });

      if (selected) {
        // 1. Update UI to show loading state
        currentDownloadPathEl.textContent = "Moving files... please wait...";

        // 2. Call Backend
        await invoke('set_downloads_path', { newPath: selected });

        // 3. Refresh Path UI
        await updateDownloadPathUI();

        // 4. Show success message with specific path
        await window.customAlert(`Downloads moved to:\n${selected}/downloads`, "Success");
      }
    } catch (e) {
      // If failed, refresh UI to show old path (or whatever state it's in)
      updateDownloadPathUI();
      await window.customAlert("Failed to set path: " + e, "Error");
    }
  });

  const openDownloadsFolderBtn = document.getElementById('openDownloadsFolderBtn');
  openDownloadsFolderBtn.addEventListener('click', async () => {
    try {
      await invoke('open_special_folder', { folderType: 'downloads' });
    } catch (e) {
      console.error(e);
    }
  });

  // Download history persistence/render/NXM handlers moved to features/downloadHistoryFeature.js

  const reorderModsByList = async (orderedModNames) => {
    try {
      // 1. Call the new Rust command, passing the desired order.
      const updatedXmlContent = await invoke('reorder_mods', { orderedModNames });

      // 2. Load the perfectly sorted XML returned by the backend. This refreshes the state.
      await loadXmlContent(updatedXmlContent, appState.currentFilePath);

      // 3. The renderModList() called by loadXmlContent will automatically redraw the UI.
      await saveChanges();
    } catch (error) {
      await window.customAlert(`Error re-ordering mods: ${error}`, "Error");
      // If it fails, re-render the original list to avoid a broken UI state
      renderModList();
    }
  };

  function reorderModListUI(orderedModNames) {
    const rowsMap = new Map();
    modListContainer.querySelectorAll('.mod-row').forEach(row => {
      rowsMap.set(row.dataset.modName, row);
    });

    orderedModNames.forEach(modName => {
      const rowElement = rowsMap.get(modName);
      if (rowElement) {
        modListContainer.appendChild(rowElement);
      }
    });

    modListContainer.querySelectorAll('.mod-row').forEach((row, index) => {
      const priorityInput = row.querySelector('.priority-input');
      if (priorityInput) {
        priorityInput.value = index;
      }
    });
  }

  const saveChanges = async () => {
    if (appState.isPopulating || !appState.currentFilePath || !appState.xmlDoc) return;
    const formattedXmlString = formatNode(appState.xmlDoc.documentElement, 0);
    const finalContent = `<?xml version="1.0" encoding="utf-8"?>\n${formattedXmlString.trimEnd()}`;
    try {
      await invoke('save_file', { filePath: appState.currentFilePath, content: finalContent });
    }
    catch (e) { await window.customAlert(`Error saving file: ${e}`, "Error"); }
  };

  const setAllModsEnabled = async (enabled) => {
    if (!appState.xmlDoc) {
      await window.customAlert("Please load a GCMODSETTINGS.MXML file first.", "Error");
      return;
    }
    const modNodes = appState.xmlDoc.querySelectorAll('Property[name="Data"] > Property[value="GcModSettingsInfo"]');
    if (modNodes.length === 0) return;

    const newValue = enabled ? 'true' : 'false';
    modNodes.forEach(modNode => {
      const enabledNode = modNode.querySelector('Property[name="Enabled"]');
      const enabledVRNode = modNode.querySelector('Property[name="EnabledVR"]');
      if (enabledNode) enabledNode.setAttribute('value', newValue);
      if (enabledVRNode) enabledVRNode.setAttribute('value', newValue);
    });

    // Save the changes to the XML in memory
    saveChanges();

    updateModListStates();
  };

  const addNewModToXml = (modName) => {
    if (!appState.xmlDoc || !modName) return;
    const dataContainer = appState.xmlDoc.querySelector('Property[name="Data"]');
    if (!dataContainer) return;

    const allMods = dataContainer.querySelectorAll('Property[value="GcModSettingsInfo"]');
    let maxIndex = -1;
    let maxPriority = -1;

    allMods.forEach(mod => {
      const index = parseInt(mod.getAttribute('_index'), 10);
      const priorityNode = mod.querySelector('Property[name="ModPriority"]');
      const priority = priorityNode ? parseInt(priorityNode.getAttribute('value'), 10) : -1;
      if (index > maxIndex) maxIndex = index;
      if (priority > maxPriority) maxPriority = priority;
    });

    const newMod = appState.xmlDoc.createElement('Property');
    newMod.setAttribute('name', 'Data');
    newMod.setAttribute('value', 'GcModSettingsInfo');
    newMod.setAttribute('_index', (maxIndex + 1).toString());

    const createProp = (name, value) => {
      const prop = appState.xmlDoc.createElement('Property');
      prop.setAttribute('name', name);
      prop.setAttribute('value', value);
      return prop;
    };

    newMod.appendChild(createProp('Name', modName.toUpperCase()));
    newMod.appendChild(createProp('Author', ''));
    newMod.appendChild(createProp('ID', '0'));
    newMod.appendChild(createProp('AuthorID', '0'));
    newMod.appendChild(createProp('LastUpdated', '0'));
    newMod.appendChild(createProp('ModPriority', (maxPriority + 1).toString()));
    newMod.appendChild(createProp('Enabled', 'true'));
    newMod.appendChild(createProp('EnabledVR', 'true'));

    const dependencies = appState.xmlDoc.createElement('Property');
    dependencies.setAttribute('name', 'Dependencies');
    newMod.appendChild(dependencies);

    dataContainer.appendChild(newMod);
  };

  // isNewerVersionAvailable moved to utils/versionUtils.js

  async function checkForAndLinkMod(modFolderName) {
    try {
      const modInfoPath = await join(appState.gamePath, 'GAMEDATA', 'MODS', modFolderName, 'mod_info.json');
      const content = await readTextFile(modInfoPath);
      const modInfo = JSON.parse(content);

      if (modInfo && modInfo.id === "") {
        const nexusUrl = await window.customPrompt(
          i18n.get('promptForNexusLink', { modName: modFolderName }),
          i18n.get('linkModTitle')
        );
        if (!nexusUrl) {
          await window.customAlert(i18n.get('linkCancelled', { modName: modFolderName }), "Cancelled");
          return;
        }
        const match = nexusUrl.match(/nexusmods\.com\/nomanssky\/mods\/(\d+)/);
        const parsedId = match ? match[1] : null;
        if (parsedId) {
          await invoke('update_mod_id_in_json', {
            modFolderName: modFolderName,
            newModId: parsedId
          });
          await window.customAlert(i18n.get('linkSuccess', { modName: modFolderName }), "Success");
        } else {
          await window.customAlert(i18n.get('linkInvalid', { modName: modFolderName }), "Error");
        }
      }
    } catch (error) { /* Silently ignore mods without info files */ }
  }

  downloadInstallFeature = createDownloadInstallFeature({
    i18n,
    invoke,
    join,
    readTextFile,
    appState,
    nexusApi,
    downloadHistoryModalOverlay,
    getDownloadHistory: () => downloadHistory,
    setDownloadHistory: (next) => { downloadHistory = next; },
    renderDownloadHistory,
    saveDownloadHistory,
    openFolderSelectionModal: (...args) => openFolderSelectionModal(...args),
    loadXmlContent: (...args) => loadXmlContent(...args),
    addNewModToXml,
    checkForAndLinkMod: (...args) => checkForAndLinkMod(...args),
    saveChanges: (...args) => saveChanges(...args),
    saveCurrentProfile: (...args) => saveCurrentProfile(...args),
    updateModDisplayState,
    logInfo: (msg) => window.addAppLog(msg, 'INFO'),
    logError: (msg) => window.addAppLog(msg, 'ERROR'),
  });

  // formatNexusDate/mapLangCode/bbcodeToHtml moved to utils modules

  // Nexus API helpers moved to features/nexusApi.js

  // --- Drag and Drop Logic (Row Reordering) ---
  function autoScrollLoop() {
    if (scrollState.isScrollingUp) modListContainer.scrollTop -= SCROLL_SPEED;
    if (scrollState.isScrollingDown) modListContainer.scrollTop += SCROLL_SPEED;
    if (dragState.draggedElement) scrollState.animationFrameId = requestAnimationFrame(autoScrollLoop);
  }

  function onMouseMove(e) {
    if (!dragState.ghostElement) return;

    dragState.ghostElement.style.left = `${e.clientX - dragState.offsetX}px`;
    dragState.ghostElement.style.top = `${e.clientY - dragState.offsetY}px`;

    const allRows = Array.from(modListContainer.querySelectorAll('.mod-row:not(.is-dragging)'));
    let nextElement = null;
    for (const row of allRows) {
      const rect = row.getBoundingClientRect();
      if (e.clientY < rect.top + rect.height / 2) {
        nextElement = row;
        break;
      }
    }
    if (nextElement) {
      modListContainer.insertBefore(dragState.placeholder, nextElement);
    } else {
      modListContainer.appendChild(dragState.placeholder);
    }

    const listRect = modListContainer.getBoundingClientRect();
    const triggerZone = 50;
    scrollState.isScrollingUp = e.clientY < listRect.top + triggerZone;
    scrollState.isScrollingDown = e.clientY > listRect.bottom - triggerZone;
  }

  function onMouseUp(e) {
    if (!dragState.draggedElement) {
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
      return;
    }

    const dropTarget = e.target.closest('#modListContainer');
    if (dropTarget && dragState.placeholder.parentNode) {
      dragState.placeholder.parentNode.insertBefore(dragState.draggedElement, dragState.placeholder);
      const finalModOrder = Array.from(modListContainer.querySelectorAll('.mod-row')).map(row => row.dataset.modName);

      window.addAppLog("User reordered mod list via drag & drop.", "INFO");

      // 1. Immediately update the UI with no blink.
      reorderModListUI(finalModOrder);

      // 2. In the background, tell the backend to save the new order.
      invoke('reorder_mods', { orderedModNames: finalModOrder })
        .then(async (updatedXmlContent) => {
          // 3. Silently update the in-memory data to match what was saved.
          appState.xmlDoc = new DOMParser().parseFromString(updatedXmlContent, "application/xml");
          await saveChanges();
          await saveCurrentProfile();
          console.log("Mod order saved and local state synced.");
        })
        .catch(async error => {
          window.addAppLog(`Failed to save reorder: ${error}`, "ERROR");
          await window.customAlert(`Error saving new mod order: ${error}`, "Error");
          renderModList();
        });
    } else {
      renderModList();
    }

    dragState.draggedElement.classList.remove('is-dragging');
    if (dragState.ghostElement) document.body.removeChild(dragState.ghostElement);
    if (dragState.placeholder) dragState.placeholder.remove();

    dragState.draggedElement = null;
    dragState.ghostElement = null;
    dragState.placeholder = null;

    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);

    cancelAnimationFrame(scrollState.animationFrameId);
    scrollState.isScrollingUp = false;
    scrollState.isScrollingDown = false;
  }

  // Browse feature handlers moved to features/browseFeature.js

  async function displayModInfo(modRow) {
    modRow.after(modInfoPanel);
    infoNexusLink.classList.add('hidden');
    infoUpdateBtn.classList.add('hidden');

    const modFolderName = modRow.dataset.modName;
    // --- Read from the in-memory cache ---
    const cachedModData = appState.modDataCache.get(modFolderName);

    if (!cachedModData) {
      console.error("Could not find data for mod in cache:", modFolderName);
      infoModName.textContent = modFolderName;
      infoDescription.textContent = "Error: Could not load mod details.";
      infoAuthor.textContent = '...';
      infoInstalledVersion.textContent = '...';
      infoLatestVersion.textContent = '...';
      modInfoPanel.classList.remove('hidden');
      return;
    }

    // Use the pre-loaded local info
    const localModInfo = cachedModData.local_info;
    if (localModInfo && localModInfo.version) {
      infoInstalledVersion.textContent = localModInfo.version;
    } else {
      infoInstalledVersion.textContent = '...';
    }

    // Now, find the remote info
    const modId = localModInfo?.mod_id;
    const remoteInfo = modId ? curatedData.find(m => String(m.mod_id) === String(modId)) : null;

    // Prioritize showing remote data, but fall back to local/default data
    infoModName.textContent = remoteInfo?.name || modFolderName;
    infoAuthor.textContent = remoteInfo?.author || 'Unknown';
    infoDescription.textContent = remoteInfo?.summary || (localModInfo ? i18n.get('noDescription') : i18n.get('noLocalInfo'));

    const updateCandidate = resolveUpdateCandidate(modFolderName, cachedModData, curatedData, { getBaseName, isNewerVersionAvailable });
    const latestVersionToShow = updateCandidate?.latestVersion || remoteInfo?.version || 'N/A';

    infoLatestVersion.textContent = latestVersionToShow;

    infoLatestVersion.classList.remove('update-available');
    if (updateCandidate) {
      infoLatestVersion.classList.add('update-available');
      infoUpdateBtn.classList.remove('hidden');
      infoUpdateBtn.dataset.modId = updateCandidate.modId;
      infoUpdateBtn.dataset.fileId = updateCandidate.latestFileId;
      infoUpdateBtn.dataset.version = updateCandidate.latestVersion;
      infoUpdateBtn.dataset.fileName = updateCandidate.latestFileName;
      infoUpdateBtn.dataset.displayName = updateCandidate.latestDisplayName;
      infoUpdateBtn.dataset.replacingFileId = updateCandidate.installedFileId || "";
    }

    if (remoteInfo?.mod_id) {
      infoNexusLink.href = `https://www.nexusmods.com/nomanssky/mods/${remoteInfo.mod_id}`;
      infoNexusLink.classList.remove('hidden');
    }

    modInfoPanel.classList.remove('hidden');
  }

  // --- Event Listeners ---

  customCloseBtn.addEventListener('click', () => appWindow.close());

  document.getElementById('minimizeBtn').addEventListener('click', () => appWindow.minimize());
  document.getElementById('maximizeBtn').addEventListener('click', async () => {
    if (await appWindow.isMaximized()) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  });

  const maximizeBtnImg = document.getElementById('maximizeBtn');
  const updateMaximizeIcon = async () => {
    const isMax = await appWindow.isMaximized();
    maximizeBtnImg.src = isMax ? iconRestore : iconMaximize;
    maximizeBtnImg.alt = isMax ? 'Restore' : 'Maximize';
    maximizeBtnImg.title = isMax ? 'Restore' : 'Maximize';
  };
  updateMaximizeIcon();
  appWindow.onResized(updateMaximizeIcon);

  const removeContextMenu = () => {
    if (contextMenu) {
      contextMenu.remove();
      contextMenu = null;
    }
  };
  const setContextMenu = (menuEl) => {
    contextMenu = menuEl;
    document.body.appendChild(menuEl);
  };

  const handleModListContextMenu = createModContextMenuHandler({
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
    saveCurrentProfile: (...args) => saveCurrentProfile(...args),
    renderModList,
    updateModDisplayState,
    getGamePath: () => appState.gamePath,
    getDownloadHistory: () => downloadHistory,
    removeContextMenu,
    setContextMenu,
  });
  window.addEventListener('click', (e) => {
    try {
      removeContextMenu(e);
    } catch (err) {
      console.error('removeContextMenu failed:', err);
      contextMenu = null;
    }
  }, true);
  window.addEventListener('contextmenu', (e) => {
    const target = e.target;
    if (target.tagName !== 'INPUT' && target.tagName !== 'TEXTAREA') {
      e.preventDefault();
    }
    removeContextMenu();
  }, true);

  modListContainer.addEventListener('contextmenu', handleModListContextMenu);

  modListContainer.addEventListener('mousedown', (e) => {
    if (e.target.closest('.switch') || e.button !== 0) return;
    const row = e.target.closest('.mod-row');
    if (!row) return;

    const modName = row.dataset.modName;

    // Check if this specific row is the ONLY one currently selected
    const isAlreadyTheSingleSelection = appState.selectedModNames.has(modName) && appState.selectedModNames.size === 1;

    // --- MULTI-SELECT LOGIC (CTRL) ---
    if (e.ctrlKey) {
      e.preventDefault();
      if (appState.selectedModNames.has(modName)) {
        appState.selectedModNames.delete(modName);
        row.classList.remove('selected');
        if (appState.selectedModRow === row) {
          appState.selectedModRow = null;
          modInfoPanel.classList.add('hidden');
        }
      } else {
        appState.selectedModNames.add(modName);
        row.classList.add('selected');
      }
      return;
    }

    // --- SINGLE SELECT LOGIC ---
    // If it's NOT the currently selected item (or there are multiple), select it immediately.
    // If it IS the currently selected item, do NOTHING yet. We wait to see if it's a Click (Toggle Off) or a Drag (Keep Selected).
    if (!isAlreadyTheSingleSelection) {
      modListContainer.querySelectorAll('.mod-row.selected').forEach(el => el.classList.remove('selected'));
      appState.selectedModNames.clear();
      appState.selectedModNames.add(modName);
      row.classList.add('selected');
    }

    // --- DRAG / CLICK HANDLING ---
    e.preventDefault();
    const DRAG_DELAY = 200;

    const handleMouseUpAsClick = () => {
      clearTimeout(dragState.dragTimer);
      document.removeEventListener('mouseup', handleMouseUpAsClick);

      // LOGIC: If we clicked the item that was ALREADY selected, we Toggle it OFF.
      if (isAlreadyTheSingleSelection) {
        appState.selectedModNames.delete(modName);
        row.classList.remove('selected');
        appState.selectedModRow = null;
        modInfoPanel.classList.add('hidden');
        return;
      }

      // Otherwise, show info
      if (appState.selectedModNames.size === 1) {
        appState.selectedModRow = row;
        displayModInfo(row);
      } else {
        modInfoPanel.classList.add('hidden');
      }
    };

    document.addEventListener('mouseup', handleMouseUpAsClick);

    dragState.dragTimer = setTimeout(() => {
      document.removeEventListener('mouseup', handleMouseUpAsClick);

      if (appState.selectedModNames.size > 1) return;

      if (appState.selectedModRow) {
        appState.selectedModRow = null;
        modInfoPanel.classList.add('hidden');
      }
      dragState.draggedElement = row;

      const rect = dragState.draggedElement.getBoundingClientRect();
      dragState.offsetX = e.clientX - rect.left;
      dragState.offsetY = e.clientY - rect.top;

      dragState.placeholder = document.createElement('div');
      dragState.placeholder.className = 'placeholder';
      dragState.placeholder.style.height = `${rect.height}px`;

      dragState.ghostElement = dragState.draggedElement.cloneNode(true);
      dragState.ghostElement.classList.add('ghost');
      document.body.appendChild(dragState.ghostElement);

      dragState.ghostElement.style.width = `${rect.width}px`;
      dragState.ghostElement.style.left = `${e.clientX - dragState.offsetX}px`;
      dragState.ghostElement.style.top = `${e.clientY - dragState.offsetY}px`;

      dragState.draggedElement.parentNode.insertBefore(dragState.placeholder, dragState.draggedElement);
      dragState.draggedElement.classList.add('is-dragging');

      scrollState.animationFrameId = requestAnimationFrame(autoScrollLoop);
      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
    }, DRAG_DELAY);
  });

  const filterModList = () => {
    const searchTerm = searchModsInput.value.trim().toLowerCase();
    const modRows = modListContainer.querySelectorAll('.mod-row');
    modRows.forEach(row => {
      const modNameElement = row.querySelector('.mod-name-text');
      if (modNameElement) {
        const modName = modNameElement.textContent.toLowerCase();
        row.style.display = modName.includes(searchTerm) ? 'flex' : 'none';
      }
    });
  };
  searchModsInput.addEventListener('input', filterModList);

  enableAllBtn.addEventListener('click', () => setAllModsEnabled(true));
  disableAllBtn.addEventListener('click', () => setAllModsEnabled(false));

  updateCheckBtn.addEventListener('click', async () => {
    await fetchCuratedData();
    await checkForUpdates(false);
  });

  closeUpdateModalBtn.addEventListener('click', () => updateModalOverlay.classList.add('hidden'));
  updateModalOverlay.addEventListener('click', (e) => {
    if (e.target === updateModalOverlay) updateModalOverlay.classList.add('hidden');
  });

  updateListContainer.addEventListener('click', async (e) => {
    const button = e.target.closest('.update-now-btn');
    if (!button) return;

    button.disabled = true;
    try {
      await startModDownload({
        modId: button.dataset.modId,
        fileId: button.dataset.fileId,
        version: button.dataset.version,
        fileName: button.dataset.fileName,
        displayName: button.dataset.displayName,
        replacingFileId: button.dataset.replacingFileId
      }, true);
    } finally {
      button.disabled = false;
    }
  });

  loadFileBtn.addEventListener('click', async () => {
    let startDir = undefined;
    if (appState.gamePath) {
      startDir = await join(appState.gamePath, 'Binaries', 'SETTINGS');
    }
    const selPath = await open({ title: i18n.get('loadFileBtn'), defaultPath: startDir, filters: [{ name: 'MXML Files', extensions: ['mxml'] }] });
    if (typeof selPath === 'string') {
      try {
        const gamePaths = await invoke('set_game_install_path', { selectedPath: selPath });
        applyDetectedGamePaths(gamePaths);
      } catch (e) {
        console.warn('Could not derive game install from selected settings file.', e);
      }
      const content = await readTextFile(selPath);
      await loadXmlContent(content, selPath);
    }
  });

  openModsFolderBtn.addEventListener('click', () => invoke('open_mods_folder'));

  disableAllSwitch.addEventListener('change', () => {
    const daNode = appState.xmlDoc.querySelector('Property[name="DisableAllMods"]');
    if (daNode) { daNode.setAttribute('value', disableAllSwitch.checked ? 'true' : 'false'); saveChanges(); }
  });

  settingsBtn.addEventListener('click', async () => {
    await updateNXMButtonState();

    document.getElementById('nxmHandlerStatus').classList.add('hidden');
    settingsModalOverlay.classList.remove('hidden');
    updateDownloadPathUI();
    updateGamePathUI(appState.gamePath);
    updateLibraryPathUI();
  });
  closeSettingsModalBtn.addEventListener('click', () => settingsModalOverlay.classList.add('hidden'));
  settingsModalOverlay.addEventListener('click', (e) => {
    if (e.target === settingsModalOverlay) settingsModalOverlay.classList.add('hidden');
  });

  const changeLibraryDirBtn = document.getElementById('changeLibraryDirBtn');
  const changeGameDirBtn = document.getElementById('changeGameDirBtn');

  changeGameDirBtn.addEventListener('click', async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select No Man\'s Sky Folder'
      });

      if (!selected) return;

      const gamePaths = await invoke('set_game_install_path', { selectedPath: selected });
      applyDetectedGamePaths(gamePaths);

      if (gamePaths.settings_initialized !== false) {
        const settingsPath = await join(gamePaths.settings_root_path, 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
        const content = await readTextFile(settingsPath);
        await loadXmlContent(content, settingsPath);
      } else {
        await window.customAlert(
          'No Man\'s Sky settings were not found yet. Launch the game once, then reopen Pulsar.',
          'Run Game First'
        );
      }
    } catch (e) {
      await window.customAlert(`Failed to set game folder: ${e}`, 'Error');
      updateGamePathUI(appState.gamePath);
    }
  });

  changeLibraryDirBtn.addEventListener('click', async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select New Library Folder"
      });

      if (selected) {
        // Show loading state because moving files might take a moment
        currentLibraryPathEl.textContent = "Moving files... please wait...";

        await invoke('set_library_path', { newPath: selected });
        await updateLibraryPathUI();

        await window.customAlert(`Library moved to:\n${selected}/Library`, "Success");
      }
    } catch (e) {
      await window.customAlert(`Failed to move library: ${e}`, "Error");
      updateLibraryPathUI();
    }
  });

  // --- SLIDER LOGIC ---

  // 1. List Density
  rowPaddingSlider.addEventListener('input', function () {
    rowPaddingValue.textContent = this.value;
    document.documentElement.style.setProperty('--mod-row-vertical-padding', `${this.value}px`);
    updateSliderFill(this);
  });
  rowPaddingSlider.addEventListener('change', function () {
    localStorage.setItem('modRowPadding', this.value);
  });

  // 2. Grid Density
  gridGapSlider.addEventListener('input', function () {
    gridGapValue.textContent = this.value;
    document.documentElement.style.setProperty('--browse-grid-gap', `${this.value}px`);
    updateSliderFill(this);
  });
  gridGapSlider.addEventListener('change', function () {
    localStorage.setItem('browseGridGap', this.value);
  });

  // 3. Mods Per Page
  modsPerPageSlider.addEventListener('input', function () {
    modsPerPageValue.textContent = this.value;
    updateSliderFill(this);
  });

  modsPerPageSlider.addEventListener('change', function () {
    const newValue = parseInt(this.value, 10);
    appState.modsPerPage = newValue;
    localStorage.setItem('modsPerPage', newValue);
    appState.currentPage = 1;
    filterAndDisplayMods();
  });

  deleteSettingsBtn.addEventListener('click', async () => {
    const confirmed = await window.customConfirm(
      i18n.get('troubleshootModalDesc'),
      i18n.get('troubleshootModalTitle')
    );
    if (!confirmed) return;
    try {
      const resultKey = await invoke('delete_settings_file');
      appState.currentFilePath = null;
      appState.xmlDoc = null;
      filePathLabel.textContent = i18n.get('noFileLoaded');
      disableAllSwitch.checked = false;
      disableAllSwitch.disabled = true;
      modListContainer.innerHTML = '';
      await window.customAlert(i18n.get(resultKey), "Success");
    } catch (error) {
      await window.customAlert(`Error: ${error}`, "Error");
    }
  });

  const cleanStagingBtn = document.getElementById('cleanStagingBtn');
  cleanStagingBtn.addEventListener('click', async () => {
    try {
      const count = await invoke('clean_staging_folder');

      if (count > 0) {
        await window.customAlert(
          i18n.get('cleanStagingSuccess', { count: count }),
          i18n.get('cleanupTitle')
        );
      } else {
        await window.customAlert(
          i18n.get('cleanStagingEmpty'),
          i18n.get('cleanupTitle')
        );
      }
    } catch (e) {
      await window.customAlert(`${i18n.get('cleanStagingError')}: ${e}`, "Error");
    }
  });

  const setupDragAndDrop = createDragDropSetup({
    appWindow,
    dropZone,
    dragState,
    appState,
    basename,
    invoke,
    i18n,
    getDownloadHistory: () => downloadHistory,
    setDownloadHistory: (next) => { downloadHistory = next; },
    renderDownloadHistory,
    openFolderSelectionModal: (...args) => openFolderSelectionModal(...args),
    processInstallAnalysis: (...args) => processInstallAnalysis(...args),
    saveDownloadHistory,
    join,
  });

  const folderSelectionModal = document.getElementById('folderSelectionModal');
  const folderSelectionList = document.getElementById('folderSelectionList');
  const fsmCancelBtn = document.getElementById('fsmCancelBtn');
  const fsmInstallAllBtn = document.getElementById('fsmInstallAllBtn');
  const fsmInstallSelectedBtn = document.getElementById('fsmInstallSelectedBtn');
  const flattenStructureCb = document.getElementById('flattenStructureCb');
  const openFolderSelectionModal = createFolderSelectionModal({
    invoke,
    folderSelectionModal,
    folderSelectionList,
    fsmCancelBtn,
    fsmInstallAllBtn,
    fsmInstallSelectedBtn,
    flattenStructureCb,
  });

  // --- PROFILE MANAGEMENT LOGIC ---

  const profileSelect = document.getElementById('profileSelect');
  const applyProfileBtn = document.getElementById('applyProfileBtn');
  const addProfileBtn = document.getElementById('addProfileBtn');
  const renameProfileBtn = document.getElementById('renameProfileBtn');
  const deleteProfileBtn = document.getElementById('deleteProfileBtn');

  // Progress Modal Elements
  const profileProgressModal = document.getElementById('profileProgressModal');
  const profileProgressText = document.getElementById('profileProgressText');
  const profileProgressBar = document.getElementById('profileProgressBar');
  const profileTimeEst = document.getElementById('profileTimeEst');

  const openProfileManagerBtn = document.getElementById('openProfileManagerBtn');
  const profileManagerModal = document.getElementById('profileManagerModal');
  const mpProfileList = document.getElementById('mpProfileList');

  // State for the modal selection
  let selectedProfileInModal = null;

  const profileFeature = createProfileFeature({
    invoke,
    appState,
    i18n,
    profileSelect,
    applyProfileBtn,
    mpProfileList,
    browseView,
    browseGridContainer,
    getDownloadHistory: () => downloadHistory,
    saveDownloadHistory,
    renderDownloadHistory,
  });

  const {
    refreshProfileList,
    renderManagerList,
    updateApplyButtonVisibility,
    getDetailedInstalledMods,
    saveCurrentProfile,
    syncDownloadHistoryWithProfile,
  } = profileFeature;

  const selectedProfileInModalRef = {
    get: () => selectedProfileInModal,
    set: (next) => { selectedProfileInModal = next; },
  };


  // Open Modal
  openProfileManagerBtn.addEventListener('click', async () => {
    selectedProfileInModal = appState.activeProfile; // Reset selection to current active
    await renderManagerList(selectedProfileInModalRef);
    profileManagerModal.classList.remove('hidden');
  });

  // Close Modal Logic
  const closeManager = () => profileManagerModal.classList.add('hidden');
  document.getElementById('mpCloseBtn').addEventListener('click', closeManager);

  const mpOpenFolderBtn = document.getElementById('mpOpenFolderBtn');
  mpOpenFolderBtn.addEventListener('click', async () => {
    try {
      await invoke('open_special_folder', { folderType: 'profiles' });
    } catch (e) {
      console.error(e);
    }
  });

  // --- MODAL ACTION BUTTONS ---

  // 1. ADD
  document.getElementById('mpCreateBtn').addEventListener('click', async () => {
    const name = await window.customPrompt(i18n.get('enterProfileName'), i18n.get('addBtn'));
    if (name && name.trim() !== "") {
      try {
        await invoke('create_empty_profile', { profileName: name });

        // Update both lists
        await renderManagerList(selectedProfileInModalRef);
        await refreshProfileList();

        // Auto-select the new one in modal
        selectedProfileInModal = name;
        renderManagerList(selectedProfileInModalRef);
      } catch (e) { await window.customAlert("Error: " + e, "Error"); }
    }
  });

  // 2. COPY
  document.getElementById('mpCopyBtn').addEventListener('click', async () => {
    if (!selectedProfileInModal) return;

    const newName = await window.customPrompt(
      i18n.get('copyProfilePrompt', { source: selectedProfileInModal }),
      i18n.get('copyBtn')
    );
    if (newName && newName.trim() !== "") {
      try {
        await invoke('copy_profile', {
          sourceName: selectedProfileInModal,
          newName: newName
        });

        await renderManagerList(selectedProfileInModalRef);
        await refreshProfileList();

        selectedProfileInModal = newName; // Select the copy
        renderManagerList(selectedProfileInModalRef);
      } catch (e) { await window.customAlert("Error copying: " + e, "Error"); }
    }
  });

  // 3. RENAME
  document.getElementById('mpRenameBtn').addEventListener('click', async () => {
    if (!selectedProfileInModal) return;
    if (selectedProfileInModal === 'Default') return await window.customAlert(i18n.get('cannotRenameDefault'), "Action Denied");

    const newName = await window.customPrompt(
      i18n.get('renameProfilePrompt', { profile: selectedProfileInModal }),
      i18n.get('renameBtn'),
      selectedProfileInModal // Pass current name as default value
    );
    if (newName && newName !== selectedProfileInModal) {
      try {
        await invoke('rename_profile', { oldName: selectedProfileInModal, newName: newName });

        // If we renamed the currently ACTIVE profile, update global state
        if (appState.activeProfile === selectedProfileInModal) {
          appState.activeProfile = newName;
          localStorage.setItem('activeProfile', newName);
          // Update main dropdown selection too
          profileSelect.value = newName;
        }

        selectedProfileInModal = newName;
        await renderManagerList(selectedProfileInModalRef);
        await refreshProfileList();
      } catch (e) { await window.customAlert("Error renaming: " + e, "Error"); }
    }
  });

  // 4. DELETE
  document.getElementById('mpRemoveBtn').addEventListener('click', async () => {
    if (!selectedProfileInModal) return;
    if (selectedProfileInModal === 'Default') return await window.customAlert(i18n.get('cannotDeleteDefault'), "Action Denied");

    if (await window.customConfirm(i18n.get('deleteProfileConfirm', { profile: selectedProfileInModal }), "Confirm")) {
      try {
        await invoke('delete_profile', { profileName: selectedProfileInModal });

        // Handle if active was deleted
        if (appState.activeProfile === selectedProfileInModal) {
          appState.activeProfile = null;
          localStorage.removeItem('activeProfile');
          profileSelect.value = 'Default';
          updateApplyButtonVisibility();
        }

        selectedProfileInModal = 'Default';
        await renderManagerList(selectedProfileInModalRef);
        await refreshProfileList();
      } catch (e) { await window.customAlert("Error deleting: " + e, "Error"); }
    }
  });

  // 5. SELECT / APPLY
  document.getElementById('mpSelectBtn').addEventListener('click', async () => {
    if (!selectedProfileInModal) return;

    closeManager();

    // Set the main dropdown to what was selected here
    profileSelect.value = selectedProfileInModal;
    updateApplyButtonVisibility();

    // Automatically trigger the Apply logic
    document.getElementById('applyProfileBtn').click();
  });

  profileSelect.addEventListener('change', updateApplyButtonVisibility);

  addProfileBtn.addEventListener('click', async () => {
    const name = await window.customPrompt(i18n.get('enterProfileName'), i18n.get('addBtn'));
    if (name && name.trim() !== "") {
      try {
        await invoke('create_empty_profile', { profileName: name });

        await refreshProfileList();

        profileSelect.value = name;
        updateApplyButtonVisibility();

      } catch (e) { await window.customAlert("Error creating profile: " + e, "Error"); }
    }
  });

  renameProfileBtn.addEventListener('click', async () => {
    const current = profileSelect.value;
    if (current === 'Default') return await window.customAlert("Cannot rename Default profile.", "Action Denied");

    const newName = await window.customPrompt(
      i18n.get('renameProfilePrompt', { profile: current }),
      i18n.get('renameBtn'),
      current
    );
    if (newName && newName !== current) {
      try {
        await invoke('rename_profile', { oldName: current, newName: newName });

        // If renamed the active profile, update the state
        if (appState.activeProfile === current) {
          appState.activeProfile = newName;
          localStorage.setItem('activeProfile', newName);
        }

        await refreshProfileList();
        profileSelect.value = newName;
        updateApplyButtonVisibility();
      } catch (e) { await window.customAlert("Error renaming: " + e, "Error"); }
    }
  });

  deleteProfileBtn.addEventListener('click', async () => {
    const current = profileSelect.value;
    if (current === 'Default') return await window.customAlert("Cannot delete Default profile.", "Action Denied");

    if (await window.customConfirm(`Delete profile "${current}"?`, "Delete Profile")) {
      try {
        await invoke('delete_profile', { profileName: current });

        // 1. Refresh the list (the deleted profile will disappear)
        await refreshProfileList();

        // 2. Check if it deleted the Active Profile
        if (appState.activeProfile === current) {
          appState.activeProfile = null;
          localStorage.removeItem('activeProfile');

          // Force dropdown to Default
          profileSelect.value = 'Default';
        } else {
          // If it deleted an inactive profile, make sure dropdown stays on the current active one
          if (appState.activeProfile) {
            profileSelect.value = appState.activeProfile;
          }
        }

        // 3. This will now show the button because ('Default' !== null)
        updateApplyButtonVisibility();

      } catch (e) { await window.customAlert("Error deleting: " + e, "Error"); }
    }
  });

  applyProfileBtn.addEventListener('click', async () => {
    const targetProfile = profileSelect.value;

    // --- SAFETY CHECK START ---
    // Prevent re-applying the profile that is already active.
    // This prevents unnecessary file operations and "purging" the folder.
    if (targetProfile === appState.activeProfile) {
      await window.customAlert(
        `The profile "${targetProfile}" is already active.`,
        "Action Ignored"
      );
      return;
    }
    // --- SAFETY CHECK END ---

    const confirmed = await window.customConfirm(
      i18n.get('switchProfileMsg', {
        profileName: targetProfile
      }),
      i18n.get('switchProfileTitle')
    );

    // If the user clicked Cancel (false), stop everything immediately.
    if (!confirmed) {
      return;
    }

    // LOGGING: Start
    window.addAppLog(`Starting Profile Switch to: ${targetProfile}`, "INFO");

    // Show Modal
    profileProgressModal.classList.remove('hidden');
    profileProgressBar.style.width = '0%';
    profileProgressText.textContent = "Initializing...";

    const start = Date.now();

    // Listen for progress from Rust
    const unlisten = await listen('profile-progress', (event) => {
      const p = event.payload;

      // Math: ((Current Mod Index - 1) * 100 + Current File %) / Total Mods
      // This gives a smooth 0-100% value for the entire process
      const totalPercentage = ((p.current - 1) * 100 + p.file_progress) / p.total;

      profileProgressBar.style.width = `${totalPercentage}%`;
      profileProgressText.textContent = `Installing ${p.current}/${p.total}: ${p.current_mod}`;

      // --- Improved Time Estimation ---
      const elapsedSeconds = (Date.now() - start) / 1000;

      // Don't estimate in the first second to avoid "Infinity" or "0s" spikes
      if (elapsedSeconds > 1 && totalPercentage > 0) {
        // Calculate speed: Percent per Second
        const rate = totalPercentage / elapsedSeconds;

        const remainingPercent = 100 - totalPercentage;
        const remainingSeconds = remainingPercent / rate;

        if (remainingSeconds < 60) {
          profileTimeEst.textContent = `Estimated time remaining: ${Math.ceil(remainingSeconds)}s`;
        } else {
          const mins = Math.ceil(remainingSeconds / 60);
          profileTimeEst.textContent = `Estimated time remaining: ~${mins} min`;
        }
      } else {
        profileTimeEst.textContent = i18n.get('calculatingTimeText');
      }
    });

    try {
      // 1. Backend swaps files
      await invoke('apply_profile', { profileName: targetProfile });

      // 2. Frontend syncs history
      await syncDownloadHistoryWithProfile(targetProfile);

      // LOGGING: Success
      window.addAppLog(`Profile Switch to ${targetProfile} successful.`, "INFO");

      // 3. Update State
      appState.activeProfile = targetProfile;
      localStorage.setItem('activeProfile', targetProfile);
      updateApplyButtonVisibility();

      // 4. Force reload of XML from disk
      const settingsPath = await join(appState.gamePath, 'Binaries', 'SETTINGS', 'GCMODSETTINGS.MXML');
      const content = await readTextFile(settingsPath);
      await loadXmlContent(content, settingsPath);

      // 5. Explicitly call refreshBrowseTabBadges AFTER renderModList finishes
      setTimeout(() => {
        refreshBrowseTabBadges();
      }, 100);

      await saveCurrentProfile();

      profileProgressModal.classList.add('hidden');
      await window.customAlert(`Profile "${targetProfile}" applied successfully.`, "Success");

    } catch (e) {
      // LOGGING: Failure
      window.addAppLog(`Profile Switch FAILED: ${e}`, "ERROR");

      profileProgressModal.classList.add('hidden');
      await window.customAlert(`Error applying profile: ${e}`, "Error");
    } finally {
      unlisten();
    }
  });

  languageSelector.addEventListener('change', (e) => i18n.loadLanguage(e.target.value));

  navMyMods.addEventListener('click', () => {
    if (isPanelOpen) modDetailCloseBtn.click();
    navMyMods.classList.add('active');
    navBrowse.classList.remove('active');
    myModsView.classList.remove('hidden');
    browseView.classList.add('hidden');
  });

  navBrowse.addEventListener('click', async () => {
    navBrowse.classList.add('active');
    navMyMods.classList.remove('active');
    browseView.classList.remove('hidden');
    myModsView.classList.add('hidden');

    // 1. Force a scan of the disk
    if (appState.activeProfile) {
      await saveCurrentProfile();
      // Update the internal map of installed mods immediately
      const installedList = await invoke('get_profile_mod_list', { profileName: appState.activeProfile });
    }

    // 2. Wait for curated data to finish loading if it hasn't already
    if (curatedDataPromise) {
      await curatedDataPromise;
    }

    if (browseGridContainer.childElementCount === 0) {
      fetchAndRenderBrowseGrid();
    } else {
      // 3. Refresh the badges on existing cards
      refreshBrowseTabBadges();
    }
  });

  browseGridContainer.addEventListener('click', (e) => {
    const previouslySelected = browseGridContainer.querySelector('.mod-card.selected');
    if (previouslySelected) {
      previouslySelected.classList.remove('selected');
    }

    const clickedCard = e.target.closest('.mod-card');
    if (clickedCard) {
      clickedCard.classList.add('selected');

      const modId = parseInt(clickedCard.dataset.modId, 10);
      const modData = curatedData.find(m => m.mod_id === modId);
      if (modData) openModDetailPanel(modData);
    }
  });

  modDetailCloseBtn.addEventListener('click', async () => {
    modDetailPanel.classList.remove('open');

    // Only shrink the window if the manager actually expanded it (PC Mode)
    if (isPanelOpen) {
      isPanelOpen = false;
      const currentSize = await appWindow.innerSize();
      await appWindow.setSize(new LogicalSize(DEFAULT_WIDTH, currentSize.height));
    }

    const currentlySelected = browseGridContainer.querySelector('.mod-card.selected');
    if (currentlySelected) {
      currentlySelected.classList.remove('selected');
    }
  });

  browseView.addEventListener('click', (e) => {
    if (isPanelOpen && !modDetailPanel.contains(e.target) && !e.target.closest('.mod-card')) {
      modDetailCloseBtn.click();
    }
  });

  closeFileSelectionModalBtn.addEventListener('click', () => fileSelectionModalOverlay.classList.add('hidden'));
  fileSelectionModalOverlay.addEventListener('click', (e) => {
    if (e.target === fileSelectionModalOverlay) fileSelectionModalOverlay.classList.add('hidden');
  });

  closeChangelogModalBtn.addEventListener('click', () => changelogModalOverlay.classList.add('hidden'));
  changelogModalOverlay.addEventListener('click', (e) => {
    if (e.target === changelogModalOverlay) changelogModalOverlay.classList.add('hidden');
  });

  const closePriorityModal = () => priorityModalOverlay.classList.add('hidden');
  cancelPriorityBtn.addEventListener('click', closePriorityModal);
  priorityModalOverlay.addEventListener('click', (e) => {
    if (e.target === priorityModalOverlay) closePriorityModal();
  });

  confirmPriorityBtn.addEventListener('click', async () => {
    const modToMove = priorityModalOverlay.dataset.modName;
    const newPriority = parseInt(priorityInput.value, 10);
    const maxPriority = parseInt(priorityInput.max, 10);
    if (isNaN(newPriority) || newPriority < 0 || newPriority > maxPriority) {
      await window.customAlert(
        i18n.get('invalidPriorityMsg', { max: maxPriority }),
        i18n.get('invalidInputTitle')
      );
      return;
    }
    let currentOrder = Array.from(modListContainer.querySelectorAll('.mod-row')).map(row => row.dataset.modName);
    currentOrder = currentOrder.filter(name => name !== modToMove);
    currentOrder.splice(newPriority, 0, modToMove);
    // 1. Immediately update the UI with no blink.
    reorderModListUI(currentOrder);

    if (appState.selectedModRow) {
      appState.selectedModRow.classList.remove('selected');
      appState.selectedModRow = null;
      modInfoPanel.classList.add('hidden');
    }

    // 2. In the background, save the changes.
    invoke('reorder_mods', { orderedModNames: currentOrder })
      .then(async (updatedXmlContent) => {
        // 3. Silently update the in-memory data.
        appState.xmlDoc = new DOMParser().parseFromString(updatedXmlContent, "application/xml");
        await saveChanges();
        await saveCurrentProfile();
        console.log("Mod order saved and local state synced.");
      })
      .catch(async error => {
        await window.customAlert(`Error saving new mod order: ${error}`, "Error");
        renderModList();
      });

    closePriorityModal()
  });

  fileSelectionListContainer.addEventListener('click', async (e) => {
    const header = e.target.closest('.collapsible-header');
    if (header) {
      header.classList.toggle('active');
      header.nextElementSibling.classList.toggle('open');
      return;
    }

    if (e.target.classList.contains('mod-card-install-btn')) {
      const button = e.target;
      const itemElement = button.closest('.update-item');

      const isUpdate = button.textContent === 'UPDATE';

      const modId = button.dataset.modId;
      const fileId = button.dataset.fileId;
      const version = button.dataset.version;
      const displayName = itemElement.querySelector('.update-item-name').textContent.split(' (v')[0];
      const rawFileName = button.dataset.rawFilename;

      // Get the ID to delete
      const replacingFileId = button.dataset.replacingFileId;

      button.disabled = true;
      fileSelectionModalOverlay.classList.add('hidden');

      await startModDownload({
        modId: modId,
        fileId: fileId,
        version: version,
        fileName: rawFileName,
        displayName: displayName,
        replacingFileId: replacingFileId
      }, isUpdate);
    }
  });

  browseSortSelect.addEventListener('input', () => {
    appState.currentPage = 1; // <--- Reset page
    filterAndDisplayMods();
  });
  browseFilterSelect.addEventListener('input', () => {
    appState.currentPage = 1; // <--- Reset page
    filterAndDisplayMods();
  });
  browseSearchInput.addEventListener('input', () => {
    appState.currentPage = 1; // <--- Reset page
    filterAndDisplayMods();
  });

  downloadHistoryBtn.addEventListener('click', async () => {
    if (appState.activeProfile) {
      await saveCurrentProfile();

      await syncDownloadHistoryWithProfile(appState.activeProfile);
    }
    renderDownloadHistory();
    downloadHistoryModalOverlay.classList.remove('hidden');
  });

  const closeDownloadHistory = () => {
    downloadHistoryModalOverlay.classList.add('hidden');
    // Clear selection when closing
    appState.selectedDownloadIds.clear();
    // Remove visual highlights
    downloadListContainer.querySelectorAll('.download-item.selected').forEach(el => el.classList.remove('selected'));
  };

  closeDownloadHistoryBtn.addEventListener('click', closeDownloadHistory);

  downloadHistoryModalOverlay.addEventListener('click', (e) => {
    if (e.target === downloadHistoryModalOverlay) {
      closeDownloadHistory();
    }
  });

  clearDownloadHistoryBtn.addEventListener('click', async () => {
    // 1. Show the new, more explicit confirmation dialog.
    const confirmed = await window.customConfirm(i18n.get('deleteAllDownloadsMsg'), i18n.get('deleteAllDownloadsTitle'));

    if (confirmed) {
      try {
        console.log("User confirmed. Deleting all downloaded archives...");

        // 2. Call the new Rust command to wipe the downloads folder.
        await invoke('clear_downloads_folder');

        // 3. Clear the in-memory history array.
        downloadHistory = [];

        // 4. Save the now-empty history array to the file.
        await saveDownloadHistory(downloadHistory);

        // 5. Re-render the UI, which will now be empty.
        renderDownloadHistory();

        console.log("All downloads successfully deleted.");

      } catch (error) {
        console.error("Failed to delete all downloads:", error);
        await window.customAlert(`An error occurred while deleting the files: ${error}`, "Error");
      }
    } else {
      console.log("User cancelled 'Delete All' operation.");
    }
  });

  document.getElementById('resetWarningsBtn').addEventListener('click', async () => {
    localStorage.removeItem('suppressUntrackedWarning');
    localStorage.removeItem('suppressInstallSuccess');
    await window.customAlert(i18n.get('resetWarningMsg'), i18n.get('resetWarningTitle'));

    await renderModList();
  });

  nxmHandlerBtn.addEventListener('click', handleNxmHandlerClick);

  document.querySelector('.download-header-row').addEventListener('click', (e) => {
    const clickedHeader = e.target.closest('.sortable');
    if (!clickedHeader) return;

    const sortKey = clickedHeader.dataset.sort;

    if (downloadSortState.key === sortKey) {
      downloadSortState.direction = downloadSortState.direction === 'asc' ? 'desc' : 'asc';
    } else {
      downloadSortState.key = sortKey;
      downloadSortState.direction = 'desc';
    }

    renderDownloadHistory();
  });

  // --- Launch Game Button ---
  const launchBtn = document.getElementById('launchGameBtn');
  const launchText = launchBtn.querySelector('.launch-text');
  const handleLaunchGameClick = createLaunchGameHandler({
    appState,
    launchBtn,
    launchText,
    i18n,
    invoke,
    addAppLog: window.addAppLog,
  });
  launchBtn.addEventListener('click', handleLaunchGameClick);

  infoUpdateBtn.addEventListener('click', async () => {
    if (!infoUpdateBtn.dataset.modId || !infoUpdateBtn.dataset.fileId) return;

    infoUpdateBtn.disabled = true;
    try {
      await startModDownload({
        modId: infoUpdateBtn.dataset.modId,
        fileId: infoUpdateBtn.dataset.fileId,
        version: infoUpdateBtn.dataset.version,
        fileName: infoUpdateBtn.dataset.fileName,
        displayName: infoUpdateBtn.dataset.displayName,
        replacingFileId: infoUpdateBtn.dataset.replacingFileId
      }, true);
    } finally {
      infoUpdateBtn.disabled = false;
    }
  });

  nexusAuthBtn.addEventListener('click', handleAuthButtonClick);

  document.getElementById('openLibraryBtn').addEventListener('click', async () => {
    try {
      await invoke('open_special_folder', { folderType: 'library' });
    } catch (e) {
      console.error(e);
    }
  });

  // --- GAMEPAD / STEAM DECK CONTROLLER SUPPORT ---
  gamepad = new GamepadManager();
  gamepad.init();

  // Define navigable sections for My Mods and Browse views
  const updateGamepadSections = () => {
    const isMyMods = !myModsView.classList.contains('hidden');
    const isBrowse = !browseView.classList.contains('hidden');

    const sections = [];

    if (isMyMods) {
      sections.push(
        { id: 'header', selector: '.nav-button, .header-button, #downloadHistoryBtn, #settingsBtn', container: 'header', promptKey: 'mods' },
        { id: 'top-panel', selector: '#disableAllSwitch, #enableAllBtn, #disableAllBtn, #loadFileBtn, #openModsFolderBtn', container: '.top-panel', promptKey: 'mods' },
        { id: 'mod-list', selector: '.mod-row', container: '#modListContainer', promptKey: 'mods' },
        { id: 'footer', selector: '#profileSelect, #applyProfileBtn:not(.hidden), .profile-btn, .profile-manage-btn, #launchGameBtn', container: '.footer-container', promptKey: 'mods' },
      );
    }

    if (isBrowse) {
      sections.push(
        { id: 'header', selector: '.nav-button, .header-button, #downloadHistoryBtn, #settingsBtn', container: 'header', promptKey: 'browse' },
        { id: 'browse-controls', selector: '#browseSearchInput, #browseFilterSelect, #browseSortSelect', container: '.browse-controls', promptKey: 'browse' },
        { id: 'browse-grid', selector: '.mod-card', container: '#browseGridContainer', promptKey: 'browse' },
        { id: 'browse-pagination', selector: '.page-btn', container: '#paginationContainer', promptKey: 'browse' },
      );
    }

    gamepad.setSections(sections);
  };

  // Update sections when switching views
  navMyMods.addEventListener('click', () => setTimeout(updateGamepadSections, 50));
  navBrowse.addEventListener('click', () => setTimeout(updateGamepadSections, 50));

  // Gamepad events
  window.addEventListener('gamepad-back', () => {
    const shouldBlurTextboxFirst = localStorage.getItem('deckBExitsTextboxFirst') !== 'false';
    const active = document.activeElement;
    const isTextEntry = active &&
      !active.readOnly &&
      !active.disabled &&
      (
        active.tagName === 'TEXTAREA' ||
        (active.tagName === 'INPUT' && ['text', 'search', 'email', 'url', 'tel', 'number', 'password'].includes((active.type || 'text').toLowerCase())) ||
        active.isContentEditable ||
        active.getAttribute?.('role') === 'textbox'
      );

    if (shouldBlurTextboxFirst && isTextEntry) {
      active.blur();
      return;
    }

    // Reuse existing Escape key logic
    const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true });
    window.dispatchEvent(event);
  });

  window.addEventListener('gamepad-launch', () => {
    const launchBtn = document.getElementById('launchGameBtn');
    if (launchBtn && !launchBtn.classList.contains('disabled')) {
      launchBtn.click();
    }
  });

  window.addEventListener('gamepad-navigate-horizontal', (e) => {
    // If focused on a mod row, toggle the enabled switch
    const focused = gamepad.focusedElement;
    if (focused && focused.classList.contains('mod-row')) {
      const toggle = focused.querySelector('.enabled-switch input[type="checkbox"]');
      if (toggle) toggle.click();
    }
  });

  // Watch for modal open/close to trap focus
  const modalObserver = new MutationObserver((mutations) => {
    for (const m of mutations) {
      if (m.type !== 'attributes' || m.attributeName !== 'class') continue;
      const el = m.target;
      if (!el.classList.contains('modal-overlay') && !el.classList.contains('mod-detail-panel')) continue;

      const wasHidden = m.oldValue?.includes('hidden');
      const isHidden = el.classList.contains('hidden');
      const wasOpen = m.oldValue?.includes('open');
      const isOpen = el.classList.contains('open');

      // Modal opened
      if ((wasHidden && !isHidden) || (!wasOpen && isOpen)) {
        gamepad.pushModalFocus(
          `#${el.id}`,
          'button, input, select, .modal-btn-confirm, .modal-btn-cancel, .modal-gen-btn-confirm, .modal-gen-btn-cancel, .mp-btn, .mp-action-btn, .download-item'
        );
      }
      // Modal closed
      if ((!wasHidden && isHidden) || (wasOpen && !isOpen)) {
        gamepad.popModalFocus();
      }
    }
  });

  // Observe all modal overlays and the detail panel
  document.querySelectorAll('.modal-overlay, .mod-detail-panel').forEach(el => {
    modalObserver.observe(el, { attributes: true, attributeOldValue: true, attributeFilter: ['class'] });
  });

  // Initial section setup (deferred to after DOM is ready)
  setTimeout(updateGamepadSections, 100);

  (async () => {
    try {
      // 1. Initialize App (Language, History, Migrations, etc.)
      await initializeApp();

      try {
        // A. Get and show version number in Settings
        const v = await getVersion();
        const el = document.getElementById('currentAppVersion');
        if (el) el.textContent = `v${v}`;

        // B. Bind the Settings Button (With Portable Check)
        const btnAppUpdate = document.getElementById('checkAppUpdateBtn');
        if (btnAppUpdate) {
          const isInstalled = await invoke('is_app_installed');

          if (isInstalled) {
            // INSTALLED MODE:
            // 1. Enable the button click listener
            btnAppUpdate.addEventListener('click', () => checkAppUpdate(true));

            // 2. Run the Silent Check immediately on startup
            checkAppUpdate(false);
          } else {
            // PORTABLE MODE:
            // 1. Change text to "PORTABLE MODE"
            btnAppUpdate.textContent = i18n.get('btnPortableMode') || "PORTABLE MODE";

            // 2. Visually disable it
            btnAppUpdate.classList.add('disabled');
            btnAppUpdate.style.opacity = "0.5";
            btnAppUpdate.style.cursor = "not-allowed";
            btnAppUpdate.title = i18n.get('portableTooltip') || "Updates disabled in Portable version";

          }
        }
      } catch (e) {
        console.warn("Failed to initialize updater UI:", e);
      }

      // 2. Initialize Profile System
      await refreshProfileList();

      const savedProfile = localStorage.getItem('activeProfile') || 'Default';
      appState.activeProfile = savedProfile;

      // Safety Check: Ensure the saved profile actually exists
      const availableProfiles = Array.from(profileSelect.options).map(opt => opt.value);
      if (availableProfiles.includes(savedProfile)) {
        profileSelect.value = savedProfile;
      } else {
        console.warn(`Saved profile '${savedProfile}' not found. Resetting to Default.`);
        profileSelect.value = 'Default';
        appState.activeProfile = 'Default';
        localStorage.setItem('activeProfile', 'Default');
      }

      updateApplyButtonVisibility();

      // 3. Initialize Drag & Drop
      await setupDragAndDrop();

      // 4. Small Screen Height Fix
      const screenHeight = window.screen.availHeight;
      const windowHeight = window.outerHeight;

      if (windowHeight > screenHeight) {
        const newHeight = Math.floor(screenHeight * 0.90);
        await appWindow.setSize(new LogicalSize(DEFAULT_WIDTH, newHeight));
        await appWindow.center();
      }

    } catch (error) {
      console.error("Critical Initialization Error:", error);
    }
  })();

  } catch (bootError) {
    console.error('Fatal startup error in main.js:', bootError);
  }
});
