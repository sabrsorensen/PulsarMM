import test from 'node:test';
import assert from 'node:assert/strict';

import { createSettingsHelpers } from '../src/features/settingsFeature.js';

function createClassList(initial = []) {
  const set = new Set(initial);
  return {
    add: (...tokens) => tokens.forEach(t => set.add(t)),
    remove: (...tokens) => tokens.forEach(t => set.delete(t)),
    contains: (token) => set.has(token),
    toArray: () => [...set],
  };
}

test('updateDownloadPathUI and updateLibraryPathUI set path text', async () => {
  const currentDownloadPathEl = { textContent: '' };
  const currentGamePathEl = { textContent: '' };
  const currentLibraryPathEl = { textContent: '' };
  const helpers = createSettingsHelpers({
    invoke: async (cmd) => {
      if (cmd === 'get_downloads_path') return '/dl';
      if (cmd === 'detect_game_installation') return { game_root_path: '/game' };
      return '/lib';
    },
    i18n: { get: (k) => k },
    currentDownloadPathEl,
    currentGamePathEl,
    currentLibraryPathEl,
  });

  await helpers.updateDownloadPathUI();
  await helpers.updateGamePathUI();
  await helpers.updateLibraryPathUI();

  assert.equal(currentDownloadPathEl.textContent, '/dl');
  assert.equal(currentGamePathEl.textContent, '/game');
  assert.equal(currentLibraryPathEl.textContent, '/lib');
});

test('path UI helpers show fallback text on errors', async () => {
  const currentDownloadPathEl = { textContent: '' };
  const currentGamePathEl = { textContent: '' };
  const currentLibraryPathEl = { textContent: '' };
  const helpers = createSettingsHelpers({
    invoke: async () => {
      throw new Error('bad');
    },
    i18n: { get: (k) => k },
    currentDownloadPathEl,
    currentGamePathEl,
    currentLibraryPathEl,
  });

  await helpers.updateDownloadPathUI();
  await helpers.updateGamePathUI();
  await helpers.updateLibraryPathUI();

  assert.equal(currentDownloadPathEl.textContent, 'Error loading path');
  assert.equal(currentGamePathEl.textContent, 'Error loading path');
  assert.equal(currentLibraryPathEl.textContent, 'Error loading path');
});

test('updateGamePathUI prefers provided game path over backend detection', async () => {
  const currentGamePathEl = { textContent: '' };
  let invoked = false;
  const helpers = createSettingsHelpers({
    invoke: async () => {
      invoked = true;
      return { game_root_path: '/detected' };
    },
    i18n: { get: (k) => k },
    currentDownloadPathEl: { textContent: '' },
    currentGamePathEl,
    currentLibraryPathEl: { textContent: '' },
  });

  await helpers.updateGamePathUI('/manual-game');

  assert.equal(currentGamePathEl.textContent, '/manual-game');
  assert.equal(invoked, false);
});

test('updateGamePathUI shows default prompt when detection returns no game path', async () => {
  const currentGamePathEl = { textContent: '' };
  const helpers = createSettingsHelpers({
    invoke: async () => null,
    i18n: { get: (k) => k },
    currentDownloadPathEl: { textContent: '' },
    currentGamePathEl,
    currentLibraryPathEl: { textContent: '' },
  });

  await helpers.updateGamePathUI();

  assert.equal(currentGamePathEl.textContent, 'Auto-detect or select a game folder.');
});

test('updateNXMButtonState updates button classes and text when registered', async () => {
  const btn = { textContent: '', className: '', classList: createClassList(['modal-btn-nxm-confirm']) };
  const statusEl = { classList: createClassList() };
  const prevDocument = global.document;
  global.document = {
    getElementById: (id) => {
      if (id === 'nxmHandlerBtn') return btn;
      if (id === 'nxmHandlerStatus') return statusEl;
      return null;
    },
  };

  try {
    const helpers = createSettingsHelpers({
      invoke: async () => true,
      i18n: { get: (k) => ({ removeHandlerBtn: 'Remove', setHandlerBtn: 'Set' }[k] || k) },
      currentDownloadPathEl: { textContent: '' },
      currentGamePathEl: { textContent: '' },
      currentLibraryPathEl: { textContent: '' },
    });

    await helpers.updateNXMButtonState();

    assert.equal(btn.textContent, 'Remove');
    assert.equal(btn.className, 'modal-btn-nxm');
    assert.equal(btn.classList.contains('modal-btn-nxm-confirm'), false);
    assert.equal(statusEl.classList.contains('hidden'), true);
  } finally {
    global.document = prevDocument;
  }
});

test('updateNXMButtonState updates button classes and text when not registered', async () => {
  const btn = { textContent: '', className: '', classList: createClassList(['modal-btn-nxm']) };
  const statusEl = { classList: createClassList() };
  const prevDocument = global.document;
  global.document = {
    getElementById: (id) => {
      if (id === 'nxmHandlerBtn') return btn;
      if (id === 'nxmHandlerStatus') return statusEl;
      return null;
    },
  };

  try {
    const helpers = createSettingsHelpers({
      invoke: async () => false,
      i18n: { get: (k) => ({ removeHandlerBtn: 'Remove', setHandlerBtn: 'Set' }[k] || k) },
      currentDownloadPathEl: { textContent: '' },
      currentGamePathEl: { textContent: '' },
      currentLibraryPathEl: { textContent: '' },
    });

    await helpers.updateNXMButtonState();

    assert.equal(btn.textContent, 'Set');
    assert.equal(btn.className, 'modal-btn-nxm-confirm');
    assert.equal(btn.classList.contains('modal-btn-nxm'), false);
    assert.equal(statusEl.classList.contains('hidden'), true);
  } finally {
    global.document = prevDocument;
  }
});

test('updateNXMButtonState handles invoke errors via catch branch', async () => {
  const btn = { textContent: '', className: '', classList: createClassList() };
  const statusEl = { classList: createClassList() };
  const prevDocument = global.document;
  const prevWarn = console.warn;
  let warned = false;
  console.warn = () => { warned = true; };
  global.document = {
    getElementById: (id) => {
      if (id === 'nxmHandlerBtn') return btn;
      if (id === 'nxmHandlerStatus') return statusEl;
      return null;
    },
  };

  try {
    const helpers = createSettingsHelpers({
      invoke: async () => { throw new Error('boom'); },
      i18n: { get: (k) => k },
      currentDownloadPathEl: { textContent: '' },
      currentGamePathEl: { textContent: '' },
      currentLibraryPathEl: { textContent: '' },
    });
    await helpers.updateNXMButtonState();
    assert.equal(warned, true);
  } finally {
    global.document = prevDocument;
    console.warn = prevWarn;
  }
});
