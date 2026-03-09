import test from 'node:test';
import assert from 'node:assert/strict';

import { createNexusAuthFeature } from '../src/features/nexusAuthFeature.js';

function createClassList(initial = []) {
  const set = new Set(initial);
  return {
    add: (...tokens) => tokens.forEach(t => set.add(t)),
    remove: (...tokens) => tokens.forEach(t => set.delete(t)),
    contains: (token) => set.has(token),
  };
}

function createFeature(overrides = {}) {
  const appState = { nexusUsername: null };
  const nexusAuthBtn = { textContent: '', className: '', style: {}, disabled: false };
  const nexusAccountStatus = { textContent: '', classList: createClassList() };
  let apiKeySet = '';

  const invoke = overrides.invoke || (async (cmd) => {
    if (cmd === 'get_nexus_api_key') return 'abc';
    if (cmd === 'http_request') return { status: 200, body: JSON.stringify({ name: 'Tester' }) };
    if (cmd === 'login_to_nexus') return 'newKey';
    if (cmd === 'logout_nexus') return true;
    throw new Error(`Unhandled command ${cmd}`);
  });

  const i18n = {
    get: (k, vars = {}) => {
      const map = {
        statusConnectedAs: `Connected as ${vars.name ?? ''}`.trim(),
        disconnectBtn: 'Disconnect',
        statusNotLoggedIn: 'Not logged in',
        connectBtn: 'Connect',
        disconnectNexusAccMsg: 'Disconnect?',
        disconnectNexusAccTitle: 'Disconnect Nexus',
      };
      return map[k] || k;
    },
  };

  const feature = createNexusAuthFeature({
    invoke,
    i18n,
    appState,
    nexusAuthBtn,
    nexusAccountStatus,
    setApiKey: (k) => {
      apiKeySet = k;
    },
  });

  return { feature, appState, nexusAuthBtn, nexusAccountStatus, getApiKey: () => apiKeySet };
}

test('validateLoginState sets logged-in UI on successful validation', async () => {
  const { feature, appState, nexusAuthBtn, nexusAccountStatus, getApiKey } = createFeature();
  const ok = await feature.validateLoginState();

  assert.equal(ok, true);
  assert.equal(getApiKey(), 'abc');
  assert.equal(appState.nexusUsername, 'Tester');
  assert.equal(nexusAccountStatus.classList.contains('logged-in'), true);
  assert.equal(nexusAuthBtn.textContent, 'Disconnect');
  assert.equal(nexusAuthBtn.className, 'modal-btn-delete');
});

test('validateLoginState sets logged-out UI on failure', async () => {
  const { feature, appState, nexusAuthBtn, nexusAccountStatus, getApiKey } = createFeature({
    invoke: async (cmd) => {
      if (cmd === 'get_nexus_api_key') return 'abc';
      if (cmd === 'http_request') return { status: 403, body: '{}' };
      throw new Error('unexpected');
    },
  });
  const ok = await feature.validateLoginState();

  assert.equal(ok, false);
  assert.equal(getApiKey(), '');
  assert.equal(appState.nexusUsername, null);
  assert.equal(nexusAccountStatus.classList.contains('logged-in'), false);
  assert.equal(nexusAuthBtn.textContent, 'Connect');
  assert.equal(nexusAuthBtn.className, 'modal-btn-confirm');
});

test('handleAuthButtonClick disconnect path logs out when confirmed', async () => {
  let didLogout = false;
  const invoke = async (cmd) => {
    if (cmd === 'logout_nexus') {
      didLogout = true;
      return true;
    }
    if (cmd === 'get_nexus_api_key') return 'abc';
    if (cmd === 'http_request') return { status: 403, body: '{}' };
    throw new Error(`Unexpected cmd ${cmd}`);
  };

  const { feature, nexusAccountStatus } = createFeature({ invoke });
  nexusAccountStatus.classList.add('logged-in');

  const prevWindow = global.window;
  global.window = {
    customConfirm: async () => true,
    customAlert: async () => {},
  };

  try {
    await feature.handleAuthButtonClick();
    assert.equal(didLogout, true);
    // validateLoginState runs after logout and should clear logged-in status.
    assert.equal(nexusAccountStatus.classList.contains('logged-in'), false);
  } finally {
    global.window = prevWindow;
  }
});

test('handleAuthButtonClick connect path calls login and success alert', async () => {
  let didLogin = false;
  let successAlert = false;
  const { feature, nexusAccountStatus, nexusAuthBtn } = createFeature({
    invoke: async (cmd) => {
      if (cmd === 'login_to_nexus') {
        didLogin = true;
        return 'newKey';
      }
      if (cmd === 'http_request') return { status: 200, body: JSON.stringify({ name: 'Alice' }) };
      if (cmd === 'get_nexus_api_key') return 'newKey';
      return true;
    },
  });
  nexusAccountStatus.classList.remove('logged-in');

  const prevWindow = global.window;
  global.window = {
    customConfirm: async () => true,
    customAlert: async (msg) => {
      if (String(msg).includes('Successfully connected')) successAlert = true;
    },
  };

  try {
    await feature.handleAuthButtonClick();
    assert.equal(didLogin, true);
    assert.equal(successAlert, true);
    assert.equal(nexusAuthBtn.disabled, false);
  } finally {
    global.window = prevWindow;
  }
});

test('handleAuthButtonClick connect path handles login failure via catch', async () => {
  let alertMessage = '';
  let validatedLoggedOut = false;
  const { feature, nexusAccountStatus, nexusAuthBtn } = createFeature({
    invoke: async (cmd) => {
      if (cmd === 'login_to_nexus') throw new Error('bad login');
      if (cmd === 'get_nexus_api_key') throw new Error('No API Key found');
      if (cmd === 'http_request') return { status: 401, body: '{}' };
      return true;
    },
  });
  nexusAccountStatus.classList.remove('logged-in');

  const prevWindow = global.window;
  global.window = {
    customConfirm: async () => true,
    customAlert: async (msg) => {
      alertMessage = String(msg);
    },
  };

  try {
    await feature.handleAuthButtonClick();
    assert.match(alertMessage, /Login Failed:/);
    assert.equal(nexusAuthBtn.disabled, false);
    validatedLoggedOut = !nexusAccountStatus.classList.contains('logged-in');
    assert.equal(validatedLoggedOut, true);
  } finally {
    global.window = prevWindow;
  }
});
