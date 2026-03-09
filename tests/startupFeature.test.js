import test from 'node:test';
import assert from 'node:assert/strict';

import { createInitializeApp } from '../src/features/startupFeature.js';

function classList(initial = []) {
  const set = new Set(initial);
  return {
    add: (...tokens) => tokens.forEach(t => set.add(t)),
    remove: (...tokens) => tokens.forEach(t => set.delete(t)),
    contains: (token) => set.has(token),
    toggle: (token, force) => {
      if (force === undefined) {
        if (set.has(token)) set.delete(token);
        else set.add(token);
        return set.has(token);
      }
      if (force) set.add(token);
      else set.delete(token);
      return force;
    },
  };
}

function makeEl() {
  return {
    textContent: '',
    value: '',
    checked: false,
    disabled: false,
    src: '',
    dataset: {},
    style: {},
    listeners: {},
    classList: classList(),
    addEventListener(type, fn) {
      this.listeners[type] = fn;
    },
    querySelector(sel) {
      if (sel === '.download-item-status') return { textContent: '' };
      if (sel === '.download-progress-bar') return { style: {}, classList: classList(['indeterminate']) };
      return null;
    },
  };
}

function setupEnv() {
  const ids = {
    autoInstallToggle: makeEl(),
    launchGameBtn: makeEl(),
    launchIcon: makeEl(),
    nxmHandlerBtn: makeEl(),
    nxmHandlerStatus: makeEl(),
  };
  const bannerText = makeEl();
  const filePathLabel = makeEl();
  const rowStatus = { textContent: '' };
  const rowBar = { style: {}, classList: classList(['indeterminate']) };
  const row = {
    querySelector: (sel) => {
      if (sel === '.download-item-status') return rowStatus;
      if (sel === '.download-progress-bar') return rowBar;
      return null;
    },
  };

  const prevDocument = global.document;
  const prevWindow = global.window;
  const prevLocalStorage = global.localStorage;

  const storage = new Map();
  global.localStorage = {
    getItem: (k) => (storage.has(k) ? storage.get(k) : null),
    setItem: (k, v) => storage.set(k, String(v)),
  };

  global.document = {
    documentElement: { style: { setProperty: () => {} } },
    getElementById: (id) => ids[id] || null,
    querySelector: (selector) => {
      if (selector === '#globalBanner .banner-text') return bannerText;
      if (selector.startsWith('.download-item[data-download-id="')) return row;
      return null;
    },
  };
  global.window = {};

  return {
    ids,
    bannerText,
    filePathLabel,
    rowStatus,
    rowBar,
    storage,
    restore() {
      global.document = prevDocument;
      global.window = prevWindow;
      global.localStorage = prevLocalStorage;
    },
  };
}

function baseDeps(overrides = {}) {
  const env = overrides.env;
  const appState = {
    gamePath: null,
    settingsPath: null,
    versionType: null,
    modsPerPage: 20,
    modDataCache: new Map(),
  };

  const rowPaddingSlider = makeEl();
  const rowPaddingValue = makeEl();
  const gridGapSlider = makeEl();
  const gridGapValue = makeEl();
  const modsPerPageSlider = makeEl();
  const modsPerPageValue = makeEl();
  const deckBExitsTextboxToggle = makeEl();
  const openModsFolderBtn = makeEl();
  const settingsBtn = makeEl();
  const updateCheckBtn = makeEl();
  const enableAllBtn = makeEl();
  const disableAllBtn = makeEl();
  const dropZone = makeEl();

  const defaultInvoke = async (cmd) => {
    if (cmd === 'run_legacy_migration') return true;
    if (cmd === 'detect_game_installation') return null;
    if (cmd === 'check_startup_intent') return null;
    if (cmd === 'check_for_untracked_mods') return false;
    if (cmd === 'get_library_path') return 'C:\\Library';
    throw new Error(`Unhandled invoke: ${cmd}`);
  };

  return {
    fetchCuratedData: async () => [],
    i18n: {
      loadLanguage: async () => {},
      updateUI: () => {},
    },
    loadDownloadHistory: async () => {},
    invoke: defaultInvoke,
    validateLoginState: async () => true,
    appState,
    rowPaddingSlider,
    rowPaddingValue,
    gridGapSlider,
    gridGapValue,
    modsPerPageSlider,
    modsPerPageValue,
    updateSliderFill: () => {},
    deckBExitsTextboxToggle,
    iconSteam: 'steam.png',
    iconGog: 'gog.png',
    iconXbox: 'xbox.png',
    openModsFolderBtn,
    settingsBtn,
    updateCheckBtn,
    enableAllBtn,
    disableAllBtn,
    dropZone,
    filePathLabel: env.filePathLabel,
    join: async (...parts) => parts.join('/'),
    readTextFile: async () => '',
    loadXmlContent: async () => {},
    listen: () => {},
    handleNxmLink: () => {},
    getDownloadHistory: () => [],
    renderModList: () => {},
    checkForUpdates: () => {},
    setCuratedDataPromise: () => {},
    ...overrides,
  };
}

test('initializeApp handles no game path and updates disabled UI state', async () => {
  const env = setupEnv();
  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('modRowPadding', '7');
    env.storage.set('browseGridGap', '11');
    env.storage.set('modsPerPage', '15');

    const deps = baseDeps({ env });
    const initializeApp = createInitializeApp(deps);
    await initializeApp();

    assert.equal(deps.openModsFolderBtn.disabled, true);
    assert.equal(deps.settingsBtn.classList.contains('disabled'), true);
    assert.equal(deps.dropZone.classList.contains('hidden'), true);
    assert.equal(env.bannerText.textContent, 'Game Not Found');
    assert.equal(deps.rowPaddingSlider.value, '7');
    assert.equal(deps.gridGapSlider.value, '11');
    assert.equal(String(deps.modsPerPageValue.textContent), '15');
  } finally {
    env.restore();
  }
});

test('initializeApp autoload failure does not write debug text into file label', async () => {
  const env = setupEnv();
  try {
    env.filePathLabel.textContent = 'No file loaded.';
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'true');
    env.storage.set('suppressUntrackedWarning', 'true');

    const deps = baseDeps({
      env,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') {
          return {
            game_root_path: 'C:\\Games\\NMS',
            settings_root_path: 'C:\\Games\\NMS',
            version_type: 'Steam',
          };
        }
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'get_library_path') return 'C:\\Library';
        if (cmd === 'check_for_untracked_mods') return false;
        throw new Error(`Unhandled ${cmd}`);
      },
      readTextFile: async () => {
        throw new Error('file not readable');
      },
      loadXmlContent: async () => {
        throw new Error('should not be called');
      },
    });

    const initializeApp = createInitializeApp(deps);
    await initializeApp();

    // Regression check for removed troubleshooting text injection.
    assert.equal(env.filePathLabel.textContent, 'No file loaded.');
  } finally {
    env.restore();
  }
});

test('initializeApp success path covers listeners, pending links, and update checks', async () => {
  const env = setupEnv();
  const warnings = [];
  const errors = [];
  const oldWarn = console.warn;
  const oldErr = console.error;
  console.warn = (...args) => warnings.push(args.join(' '));
  console.error = (...args) => errors.push(args.join(' '));

  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('modRowPadding', '6');
    env.storage.set('browseGridGap', '9');
    env.storage.set('modsPerPage', '22');
    env.storage.set('deckBExitsTextboxFirst', 'false');
    env.storage.set('autoInstallAfterDownload', 'true');
    env.storage.set('suppressDriveCheck', 'false');
    env.storage.set('suppressUntrackedWarning', 'false');

    const listenHandlers = new Map();
    let loadXmlCalled = 0;
    let renderModListCalled = 0;
    let checkForUpdatesCalled = 0;
    let handleNxmLinkCalled = '';
    let updateUICalled = 0;
    let autoInstallStored = '';
    let deckStored = '';

    const appState = {
      gamePath: null,
      settingsPath: null,
      versionType: null,
      modsPerPage: 20,
      modDataCache: new Map([['x', {}]]),
    };

    const deps = baseDeps({
      env,
      appState,
      fetchCuratedData: async () => [{ mod_id: 1 }],
      i18n: {
        loadLanguage: async () => {},
        updateUI: () => { updateUICalled += 1; },
      },
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') {
          return {
            game_root_path: 'D:\\Games\\NMS',
            settings_root_path: 'D:\\Games\\NMS',
            version_type: 'GOG',
          };
        }
        if (cmd === 'get_library_path') return 'C:\\Library';
        if (cmd === 'check_for_untracked_mods') return true;
        if (cmd === 'check_startup_intent') return 'nxm://nomanssky/mods/1/files/2';
        return true;
      },
      readTextFile: async () => '<xml>valid content</xml>',
      loadXmlContent: async () => { loadXmlCalled += 1; },
      listen: (event, cb) => { listenHandlers.set(event, cb); },
      getDownloadHistory: () => [{ id: 'dl-1', statusText: '' }],
      handleNxmLink: (link) => { handleNxmLinkCalled = link; },
      renderModList: () => { renderModListCalled += 1; },
      checkForUpdates: () => { checkForUpdatesCalled += 1; },
    });

    const initializeApp = createInitializeApp(deps);
    await initializeApp();

    assert.equal(deps.openModsFolderBtn.disabled, false);
    assert.equal(deps.settingsBtn.classList.contains('disabled'), false);
    assert.equal(deps.dropZone.classList.contains('hidden'), false);
    assert.equal(env.ids.launchIcon.src, 'gog.png');
    assert.equal(loadXmlCalled, 1);
    assert.equal(renderModListCalled, 1); // untracked mods branch
    assert.equal(checkForUpdatesCalled, 1);
    assert.equal(updateUICalled, 1);
    assert.equal(handleNxmLinkCalled, 'nxm://nomanssky/mods/1/files/2');
    assert.ok(warnings.some(w => w.includes('Library/Game drive mismatch')));
    assert.ok(warnings.some(w => w.includes('Untracked mods detected')));

    // Trigger install-progress listener branch.
    const progressCb = listenHandlers.get('install-progress');
    assert.ok(progressCb);
    progressCb({ payload: { id: 'dl-1', step: 'Downloading', progress: 42 } });
    assert.equal(env.rowStatus.textContent, 'Downloading');
    assert.equal(env.rowBar.style.width, '42%');
    assert.equal(env.rowBar.classList.contains('indeterminate'), false);
    progressCb({ payload: { id: 'dl-1', step: 'Extracting' } });
    assert.equal(env.rowStatus.textContent, 'Extracting');
    assert.equal(env.rowBar.style.width, '100%');
    assert.equal(env.rowBar.style.opacity, '0.5');

    const nxmCb = listenHandlers.get('nxm-link-received');
    assert.ok(nxmCb);
    nxmCb({ payload: 'nxm://nomanssky/mods/2/files/3' });
    assert.equal(handleNxmLinkCalled, 'nxm://nomanssky/mods/2/files/3');

    // Trigger toggle listeners to cover localStorage set paths.
    deps.modsPerPageSlider.value = 22;
    env.ids.autoInstallToggle.checked = false;
    env.ids.autoInstallToggle.listeners.change.call(env.ids.autoInstallToggle);
    deps.deckBExitsTextboxToggle.checked = true;
    deps.deckBExitsTextboxToggle.listeners.change.call(deps.deckBExitsTextboxToggle);

    autoInstallStored = env.storage.get('autoInstallAfterDownload');
    deckStored = env.storage.get('deckBExitsTextboxFirst');
    assert.equal(autoInstallStored, 'false');
    assert.equal(deckStored, 'true');
  } finally {
    console.warn = oldWarn;
    console.error = oldErr;
    env.restore();
  }
});

test('initializeApp sets GamePass icon branch', async () => {
  const env = setupEnv();
  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'true');
    env.storage.set('suppressUntrackedWarning', 'true');

    const deps = baseDeps({
      env,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') {
          return {
            game_root_path: 'E:\\Games\\NMS',
            settings_root_path: 'E:\\Games\\NMS',
            version_type: 'GamePass',
          };
        }
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'get_library_path') return 'E:\\Library';
        if (cmd === 'check_for_untracked_mods') return false;
        return true;
      },
      readTextFile: async () => '<xml>valid</xml>',
    });
    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    assert.equal(env.ids.launchIcon.src, 'xbox.png');
  } finally {
    env.restore();
  }
});

test('initializeApp handles startup intent check errors gracefully', async () => {
  const env = setupEnv();
  const errors = [];
  const oldErr = console.error;
  console.error = (...args) => errors.push(args.join(' '));

  try {
    env.storage.set('selectedLanguage', 'en');
    const deps = baseDeps({
      env,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') return null;
        if (cmd === 'check_startup_intent') throw new Error('intent fail');
        return false;
      },
    });
    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    assert.ok(errors.some(e => e.includes('Failed to check startup intent')));
  } finally {
    console.error = oldErr;
    env.restore();
  }
});

test('initializeApp handles migration promise rejection without failing startup', async () => {
  const env = setupEnv();
  const errors = [];
  const oldErr = console.error;
  console.error = (...args) => errors.push(args.join(' '));

  try {
    env.storage.set('selectedLanguage', 'en');
    const deps = baseDeps({
      env,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') throw new Error('migration fail');
        if (cmd === 'detect_game_installation') return null;
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'check_for_untracked_mods') return false;
        return true;
      },
    });
    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    assert.ok(errors.some(e => e.includes('Migration error:')));
  } finally {
    console.error = oldErr;
    env.restore();
  }
});

test('initializeApp startup listeners handle non-matching progress rows and null drive paths', async () => {
  const env = setupEnv();
  const warnings = [];
  const oldWarn = console.warn;
  console.warn = (...args) => warnings.push(args.join(' '));

  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'false');
    env.storage.set('suppressUntrackedWarning', 'true');

    const listenHandlers = new Map();
    const appState = {
      gamePath: '/linux/path/without-drive',
      settingsPath: null,
      versionType: null,
      modsPerPage: 20,
      modDataCache: new Map(),
    };

    const deps = baseDeps({
      env,
      appState,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') {
          return {
            game_root_path: '/linux/path/without-drive',
            settings_root_path: null,
            version_type: 'Steam',
          };
        }
        if (cmd === 'get_library_path') return '/another/path/without-drive';
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'check_for_untracked_mods') return false;
        return true;
      },
      listen: (event, cb) => { listenHandlers.set(event, cb); },
      getDownloadHistory: () => [],
      readTextFile: async () => '<xml>valid</xml>',
      modsPerPageSlider: null,
      modsPerPageValue: null,
    });

    const initializeApp = createInitializeApp(deps);
    await initializeApp();

    const progressCb = listenHandlers.get('install-progress');
    assert.ok(progressCb);
    const oldQuerySelector = global.document.querySelector;
    global.document.querySelector = (selector) => {
      if (selector.startsWith('.download-item[data-download-id="')) return null;
      return oldQuerySelector(selector);
    };
    progressCb({ payload: { id: 'missing-id', step: 'Ignored', progress: 10 } });
    global.document.querySelector = oldQuerySelector;
    assert.equal(warnings.length, 0);
  } finally {
    console.warn = oldWarn;
    env.restore();
  }
});

test('initializeApp install-progress listener tolerates missing status and progress elements', async () => {
  const env = setupEnv();
  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'true');
    env.storage.set('suppressUntrackedWarning', 'true');

    const listenHandlers = new Map();
    const row = { querySelector: () => null };
    const oldQuerySelector = global.document.querySelector;
    global.document.querySelector = (selector) => {
      if (selector.startsWith('.download-item[data-download-id="')) return row;
      return oldQuerySelector(selector);
    };

    const deps = baseDeps({
      env,
      listen: (event, cb) => { listenHandlers.set(event, cb); },
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') return null;
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'check_for_untracked_mods') return false;
        return true;
      },
      getDownloadHistory: () => [{ id: 'dl-x', statusText: '' }],
    });

    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    const progressCb = listenHandlers.get('install-progress');
    assert.ok(progressCb);
    progressCb({ payload: { id: 'dl-x', step: 'Working', progress: null } });

    global.document.querySelector = oldQuerySelector;
  } finally {
    env.restore();
  }
});

test('initializeApp checks untracked mods with false result when warning not suppressed', async () => {
  const env = setupEnv();
  let renderModListCalled = 0;
  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'true');
    env.storage.set('suppressUntrackedWarning', 'false');

    const deps = baseDeps({
      env,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') return null;
        if (cmd === 'check_for_untracked_mods') return false;
        if (cmd === 'check_startup_intent') return null;
        return true;
      },
      renderModList: () => { renderModListCalled += 1; },
    });
    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    assert.equal(renderModListCalled, 0);
  } finally {
    env.restore();
  }
});

test('initializeApp uses localStorage defaults and skips xml load for short content', async () => {
  const env = setupEnv();
  let sliderFillCalls = 0;
  let loadXmlCalled = 0;
  try {
    const deps = baseDeps({
      env,
      updateSliderFill: () => { sliderFillCalls += 1; },
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') {
          return {
            game_root_path: 'C:\\Games\\NMS',
            settings_root_path: 'C:\\Games\\NMS',
            version_type: 'Steam',
          };
        }
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'check_for_untracked_mods') return false;
        if (cmd === 'get_library_path') return 'C:\\Games\\Lib';
        return true;
      },
      readTextFile: async () => 'short',
      loadXmlContent: async () => { loadXmlCalled += 1; },
    });

    const initializeApp = createInitializeApp(deps);
    await initializeApp();

    assert.equal(deps.rowPaddingSlider.value, '5');
    assert.equal(deps.gridGapSlider.value, '10');
    assert.equal(deps.modsPerPageSlider.value, 20);
    assert.equal(sliderFillCalls >= 3, true);
    assert.equal(loadXmlCalled, 0);
  } finally {
    env.restore();
  }
});

test('initializeApp handles empty library path in drive detection helper', async () => {
  const env = setupEnv();
  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'false');
    env.storage.set('suppressUntrackedWarning', 'true');

    const deps = baseDeps({
      env,
      invoke: async (cmd) => {
        if (cmd === 'run_legacy_migration') return true;
        if (cmd === 'detect_game_installation') {
          return {
            game_root_path: 'C:\\Games\\NMS',
            settings_root_path: null,
            version_type: 'Steam',
          };
        }
        if (cmd === 'get_library_path') return '';
        if (cmd === 'check_startup_intent') return null;
        if (cmd === 'check_for_untracked_mods') return false;
        return true;
      },
    });

    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    assert.equal(deps.openModsFolderBtn.disabled, false);
  } finally {
    env.restore();
  }
});

test('initializeApp warns once when settings file is not initialized', async () => {
  const env = setupEnv();
  let alertCount = 0;
  let loadXmlCalled = 0;
  const prevWindow = global.window;
  global.window = {
    customAlert: async () => {
      alertCount += 1;
    },
  };

  try {
    env.storage.set('selectedLanguage', 'en');
    env.storage.set('suppressDriveCheck', 'true');
    env.storage.set('suppressUntrackedWarning', 'true');

    const invoke = async (cmd) => {
      if (cmd === 'run_legacy_migration') return true;
      if (cmd === 'detect_game_installation') {
        return {
          game_root_path: '/home/deck/.steam/steam/steamapps/common/No Man\'s Sky',
          settings_root_path: '/home/deck/.steam/steam/steamapps/common/No Man\'s Sky',
          version_type: 'Steam',
          settings_initialized: false,
        };
      }
      if (cmd === 'check_startup_intent') return null;
      if (cmd === 'check_for_untracked_mods') return false;
      if (cmd === 'get_library_path') return '/home/deck/.local/share/pulsar/Library';
      return true;
    };

    const deps = baseDeps({
      env,
      invoke,
      readTextFile: async () => '<xml>valid</xml>',
      loadXmlContent: async () => { loadXmlCalled += 1; },
    });
    const initializeApp = createInitializeApp(deps);
    await initializeApp();
    await initializeApp();

    assert.equal(alertCount, 1);
    assert.equal(loadXmlCalled, 0);
  } finally {
    global.window = prevWindow;
    env.restore();
  }
});
