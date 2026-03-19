import test from 'node:test';
import assert from 'node:assert/strict';

import { createDownloadInstallFeature } from '../src/features/downloadInstallFeature.js';

function makeDeps(overrides = {}) {
  const downloadHistory = [];

  return {
    i18n: {
      get: (key) => key,
    },
    invoke: async (cmd) => {
      if (cmd === 'ensure_mod_info') return;
      throw new Error(`Unhandled invoke: ${cmd}`);
    },
    join: async (...parts) => parts.join('/'),
    readTextFile: async () => '',
    appState: {
      modDataCache: new Map(),
      currentFilePath: '/tmp/GCMODSETTINGS.MXML',
    },
    nexusApi: {},
    downloadHistoryModalOverlay: { classList: { remove() {} } },
    getDownloadHistory: () => downloadHistory,
    setDownloadHistory: () => {},
    renderDownloadHistory: async () => {},
    saveDownloadHistory: async () => {},
    openFolderSelectionModal: async () => null,
    loadXmlContent: async () => {},
    renderModList: async () => {},
    addNewModToXml: () => {},
    checkForAndLinkMod: async () => {},
    saveChanges: async () => {},
    saveCurrentProfile: async () => {},
    updateModDisplayState: () => {},
    logInfo: () => {},
    logError: () => {},
    ...overrides,
  };
}

test('processInstallAnalysis refreshes mod list after a successful install', async () => {
  const calls = [];
  const item = {
    id: 'download-1',
    modId: '',
    fileId: '',
    version: '1.0',
    fileName: 'Example.zip',
    statusText: 'statusWaiting',
    statusClass: 'progress',
    archivePath: '/tmp/Example.zip',
    modFolderName: null,
  };

  const deps = makeDeps({
    getDownloadHistory: () => [item],
    renderModList: async () => {
      calls.push('renderModList');
    },
    saveChanges: async () => {
      calls.push('saveChanges');
    },
    renderDownloadHistory: async () => {
      calls.push('renderDownloadHistory');
    },
    saveCurrentProfile: async () => {
      calls.push('saveCurrentProfile');
    },
    saveDownloadHistory: async () => {
      calls.push('saveDownloadHistory');
    },
  });

  const feature = createDownloadInstallFeature(deps);

  await feature.processInstallAnalysis(
    {
      conflicts: [],
      successes: [{ name: 'InstalledMod' }],
    },
    item,
    false
  );

  assert.deepEqual(
    calls,
    [
      'saveChanges',
      'renderModList',
      'renderDownloadHistory',
      'renderDownloadHistory',
      'saveDownloadHistory',
      'saveCurrentProfile',
    ]
  );
  assert.equal(item.modFolderName, 'InstalledMod');
  assert.equal(item.statusClass, 'installed');
});
