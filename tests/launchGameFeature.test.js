import test from 'node:test';
import assert from 'node:assert/strict';

import { createLaunchGameHandler } from '../src/features/launchGameFeature.js';

function createClassList(initial = []) {
  const set = new Set(initial);
  return {
    add: (...tokens) => tokens.forEach(t => set.add(t)),
    remove: (...tokens) => tokens.forEach(t => set.delete(t)),
    contains: (token) => set.has(token),
  };
}

test('returns early when game path/version missing', async () => {
  const appState = { gamePath: null, versionType: null };
  const launchBtn = { classList: createClassList() };
  const launchText = { textContent: 'Launch' };
  let invokeCalled = false;

  const handleLaunchGameClick = createLaunchGameHandler({
    appState,
    launchBtn,
    launchText,
    i18n: { get: () => 'Launching...' },
    invoke: async () => {
      invokeCalled = true;
    },
    addAppLog: () => {},
  });

  await handleLaunchGameClick();
  assert.equal(invokeCalled, false);
});

test('successful launch triggers invoke and resets state after timeout', async () => {
  const appState = { gamePath: '/game', versionType: 'Steam' };
  const launchBtn = { classList: createClassList() };
  const launchText = { textContent: 'Launch' };
  const logs = [];
  const timeouts = [];

  const prevSetTimeout = global.setTimeout;
  global.setTimeout = (fn, ms) => {
    timeouts.push(ms);
    fn();
    return 0;
  };

  try {
    const handleLaunchGameClick = createLaunchGameHandler({
      appState,
      launchBtn,
      launchText,
      i18n: { get: () => 'Launching...' },
      invoke: async (cmd, payload) => {
        assert.equal(cmd, 'launch_game');
        assert.deepEqual(payload, { versionType: 'Steam', gamePath: '/game' });
        return 'shell';
      },
      addAppLog: (msg, level) => logs.push({ msg, level }),
    });

    await handleLaunchGameClick();
    assert.equal(launchBtn.classList.contains('is-launching'), false);
    assert.equal(launchText.textContent, 'Launch');
    assert.deepEqual(timeouts, [10000]);
    assert.equal(logs[0].level, 'INFO');
    assert.match(logs[0].msg, /Game launch command sent/);
  } finally {
    global.setTimeout = prevSetTimeout;
  }
});

test('launch failure logs error and shows alert', async () => {
  const appState = { gamePath: '/game', versionType: 'Steam' };
  const launchBtn = { classList: createClassList() };
  const launchText = { textContent: 'Launch' };
  const logs = [];
  let alertMessage = '';

  const prevWindow = global.window;
  global.window = {
    customAlert: async (msg) => {
      alertMessage = String(msg);
    },
  };

  try {
    const handleLaunchGameClick = createLaunchGameHandler({
      appState,
      launchBtn,
      launchText,
      i18n: { get: () => 'Launching...' },
      invoke: async () => {
        throw new Error('cannot launch');
      },
      addAppLog: (msg, level) => logs.push({ msg, level }),
    });

    await handleLaunchGameClick();
    assert.equal(launchBtn.classList.contains('is-launching'), false);
    assert.equal(launchText.textContent, 'Launch');
    assert.equal(logs[0].level, 'ERROR');
    assert.match(alertMessage, /Failed to launch game/);
  } finally {
    global.window = prevWindow;
  }
});

test('returns early when launch is already in progress', async () => {
  const appState = { gamePath: '/game', versionType: 'Steam' };
  const launchBtn = { classList: createClassList(['is-launching']) };
  const launchText = { textContent: 'Launch' };
  let invokeCalled = false;

  const handleLaunchGameClick = createLaunchGameHandler({
    appState,
    launchBtn,
    launchText,
    i18n: { get: () => 'Launching...' },
    invoke: async () => {
      invokeCalled = true;
      return true;
    },
    addAppLog: () => {},
  });

  await handleLaunchGameClick();
  assert.equal(invokeCalled, false);
});
