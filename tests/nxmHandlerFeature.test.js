import test from 'node:test';
import assert from 'node:assert/strict';

import { createNxmHandlerFeature } from '../src/features/nxmHandlerFeature.js';

function createClassList(initial = []) {
  const set = new Set(initial);
  return {
    add: (...tokens) => tokens.forEach(t => set.add(t)),
    remove: (...tokens) => tokens.forEach(t => set.delete(t)),
    contains: (token) => set.has(token),
  };
}

test('registered + confirm=true unregisters and shows success', async () => {
  const nxmHandlerBtn = { disabled: false };
  let updateCalls = 0;
  const statusEl = { textContent: '', className: '', classList: createClassList(['hidden']) };
  const invokeCalls = [];

  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = { getElementById: () => statusEl };
  global.window = { customConfirm: async () => true };

  try {
    const feature = createNxmHandlerFeature({
      nxmHandlerBtn,
      i18n: { get: (k) => ({ removeNXMHandlerMsg: 'rm?', removeNXMHandlerTitle: 'rm', nxmRemovedSuccess: 'Removed!' }[k] || k) },
      invoke: async (cmd) => {
        invokeCalls.push(cmd);
        if (cmd === 'is_protocol_handler_registered') return true;
        return true;
      },
      updateNXMButtonState: async () => { updateCalls += 1; },
    });

    await feature.handleNxmHandlerClick();

    assert.deepEqual(invokeCalls.slice(0, 2), ['is_protocol_handler_registered', 'unregister_nxm_protocol']);
    assert.equal(updateCalls, 1);
    assert.equal(statusEl.textContent, 'Removed!');
    assert.equal(statusEl.className, 'handler-status status-success');
    assert.equal(statusEl.classList.contains('hidden'), false);
    assert.equal(nxmHandlerBtn.disabled, false);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('registered + confirm=false exits early and re-enables button', async () => {
  const nxmHandlerBtn = { disabled: false };
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = { getElementById: () => ({ textContent: '', className: '', classList: createClassList(['hidden']) }) };
  global.window = { customConfirm: async () => false };

  let invokeCount = 0;
  try {
    const feature = createNxmHandlerFeature({
      nxmHandlerBtn,
      i18n: { get: (k) => k },
      invoke: async (cmd) => {
        invokeCount += 1;
        if (cmd === 'is_protocol_handler_registered') return true;
        return true;
      },
      updateNXMButtonState: async () => {},
    });
    await feature.handleNxmHandlerClick();
    assert.equal(invokeCount, 1);
    assert.equal(nxmHandlerBtn.disabled, false);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('not registered path registers and handles thrown errors', async () => {
  const nxmHandlerBtn = { disabled: false };
  const statusEl = { textContent: '', className: '', classList: createClassList(['hidden']) };
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = { getElementById: () => statusEl };
  global.window = { customConfirm: async () => true };

  try {
    const feature = createNxmHandlerFeature({
      nxmHandlerBtn,
      i18n: { get: (k) => ({ addNXMHandlerMsg: 'add?', addNXMHandlerTitle: 'add', nxmSetSuccess: 'Set!' }[k] || k) },
      invoke: async (cmd) => {
        if (cmd === 'is_protocol_handler_registered') return false;
        if (cmd === 'register_nxm_protocol') throw new Error('boom');
        return true;
      },
      updateNXMButtonState: async () => {},
    });

    await feature.handleNxmHandlerClick();
    assert.match(statusEl.textContent, /Error:/);
    assert.equal(statusEl.className, 'handler-status status-error');
    assert.equal(nxmHandlerBtn.disabled, false);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('not registered + confirm=true registers and reports success', async () => {
  const nxmHandlerBtn = { disabled: false };
  const statusEl = { textContent: '', className: '', classList: createClassList(['hidden']) };
  const prevDocument = global.document;
  const prevWindow = global.window;
  const invokeCalls = [];
  let updateCalls = 0;
  global.document = { getElementById: () => statusEl };
  global.window = { customConfirm: async () => true };

  try {
    const feature = createNxmHandlerFeature({
      nxmHandlerBtn,
      i18n: { get: (k) => ({ addNXMHandlerMsg: 'add?', addNXMHandlerTitle: 'add', nxmSetSuccess: 'Set!' }[k] || k) },
      invoke: async (cmd) => {
        invokeCalls.push(cmd);
        if (cmd === 'is_protocol_handler_registered') return false;
        return true;
      },
      updateNXMButtonState: async () => { updateCalls += 1; },
    });

    await feature.handleNxmHandlerClick();

    assert.deepEqual(invokeCalls.slice(0, 2), ['is_protocol_handler_registered', 'register_nxm_protocol']);
    assert.equal(updateCalls, 1);
    assert.equal(statusEl.textContent, 'Set!');
    assert.equal(statusEl.className, 'handler-status status-success');
    assert.equal(nxmHandlerBtn.disabled, false);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('not registered + confirm=false exits without register and re-enables button', async () => {
  const nxmHandlerBtn = { disabled: false };
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = { getElementById: () => ({ textContent: '', className: '', classList: createClassList(['hidden']) }) };
  global.window = { customConfirm: async () => false };

  const invokeCalls = [];
  try {
    const feature = createNxmHandlerFeature({
      nxmHandlerBtn,
      i18n: { get: (k) => k },
      invoke: async (cmd) => {
        invokeCalls.push(cmd);
        if (cmd === 'is_protocol_handler_registered') return false;
        return true;
      },
      updateNXMButtonState: async () => {},
    });
    await feature.handleNxmHandlerClick();
    assert.deepEqual(invokeCalls, ['is_protocol_handler_registered']);
    assert.equal(nxmHandlerBtn.disabled, false);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('uses fallback success messages when i18n keys are missing', async () => {
  const nxmHandlerBtn = { disabled: false };
  const statusEl = { textContent: '', className: '', classList: createClassList(['hidden']) };
  const prevDocument = global.document;
  const prevWindow = global.window;

  const confirms = [true, true];
  let isRegistered = true;
  global.document = { getElementById: () => statusEl };
  global.window = { customConfirm: async () => confirms.shift() };

  try {
    const feature = createNxmHandlerFeature({
      nxmHandlerBtn,
      i18n: { get: () => undefined },
      invoke: async (cmd) => {
        if (cmd === 'is_protocol_handler_registered') return isRegistered;
        if (cmd === 'unregister_nxm_protocol') {
          isRegistered = false;
          return true;
        }
        if (cmd === 'register_nxm_protocol') {
          isRegistered = true;
          return true;
        }
        return true;
      },
      updateNXMButtonState: async () => {},
    });

    await feature.handleNxmHandlerClick();
    assert.equal(statusEl.textContent, 'Successfully removed.');
    await feature.handleNxmHandlerClick();
    assert.equal(statusEl.textContent, 'Successfully set!');
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});
