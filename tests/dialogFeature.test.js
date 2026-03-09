import test from 'node:test';
import assert from 'node:assert/strict';

import { installDialogHelpers } from '../src/features/dialogFeature.js';

function classList(initial = []) {
  const set = new Set(initial);
  return {
    add: (...tokens) => tokens.forEach(t => set.add(t)),
    remove: (...tokens) => tokens.forEach(t => set.delete(t)),
    contains: (token) => set.has(token),
  };
}

function makeElement(opts = {}) {
  let _innerHTML = '';
  const el = {
    textContent: '',
    value: '',
    onclick: null,
    onkeydown: null,
    style: {},
    className: '',
    classList: classList(opts.hidden ? ['hidden'] : []),
    children: [],
    focusCalled: false,
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    focus() {
      this.focusCalled = true;
    },
  };
  Object.defineProperty(el, 'innerHTML', {
    get() {
      return _innerHTML;
    },
    set(v) {
      _innerHTML = String(v);
      // Mimic DOM behavior used by dialogFeature when resetting actions.
      if (_innerHTML === '') {
        this.children = [];
      }
    },
  });
  return el;
}

function setupDialogDom() {
  const map = {
    inputDialogModal: makeElement({ hidden: true }),
    inputDialogTitle: makeElement(),
    inputDialogMessage: makeElement(),
    inputDialogField: makeElement(),
    inputDialogCancelBtn: makeElement(),
    inputDialogOkBtn: makeElement(),
    genericDialogModal: makeElement({ hidden: true }),
    genericDialogTitle: makeElement(),
    genericDialogMessage: makeElement(),
    genericDialogActions: makeElement(),
  };

  const documentStub = {
    getElementById(id) {
      return map[id];
    },
    createElement(tag) {
      assert.equal(tag, 'button');
      return makeElement();
    },
  };

  return { map, documentStub };
}

test('customPrompt resolves entered value and toggles modal classes', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  const prevSetTimeout = global.setTimeout;
  global.document = documentStub;
  global.window = {};
  global.setTimeout = (fn) => {
    fn();
    return 0;
  };

  try {
    installDialogHelpers({ getText: (k) => k });
    const promptPromise = window.customPrompt('Message', 'Title', 'default');
    map.inputDialogField.value = 'chosen';
    map.inputDialogOkBtn.onclick();
    const value = await promptPromise;

    assert.equal(value, 'chosen');
    assert.equal(map.inputDialogModal.classList.contains('hidden'), true);
    assert.equal(map.inputDialogField.focusCalled, true);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
    global.setTimeout = prevSetTimeout;
  }
});

test('customPrompt resolves null on escape key', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  const prevSetTimeout = global.setTimeout;
  global.document = documentStub;
  global.window = {};
  global.setTimeout = (fn) => {
    fn();
    return 0;
  };

  try {
    installDialogHelpers({ getText: (k) => k });
    const promptPromise = window.customPrompt('Message', 'Title', 'default');
    map.inputDialogField.onkeydown({ key: 'Escape' });
    const value = await promptPromise;
    assert.equal(value, null);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
    global.setTimeout = prevSetTimeout;
  }
});

test('customConfirm renders cancel/ok and resolves false on cancel', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = documentStub;
  global.window = {};

  try {
    installDialogHelpers({ getText: (k) => (k === 'okBtn' ? 'OK' : 'Cancel') });
    const p = window.customConfirm('Are you sure?', 'Confirm', null, 'Do Not Proceed');

    assert.equal(map.genericDialogModal.classList.contains('hidden'), false);
    assert.equal(map.genericDialogActions.children.length, 2);
    const cancelBtn = map.genericDialogActions.children[0];
    assert.equal(cancelBtn.style.width, 'auto');
    cancelBtn.onclick();

    const result = await p;
    assert.equal(result, false);
    assert.equal(map.genericDialogModal.classList.contains('hidden'), true);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('customConflictDialog resolves selected action', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = documentStub;
  global.window = {};

  try {
    installDialogHelpers({ getText: (k) => k });
    const p = window.customConflictDialog('Conflict', 'Title', 'Replace', 'Keep', 'Cancel');
    assert.equal(map.genericDialogActions.children.length, 3);
    const replaceBtn = map.genericDialogActions.children[2];
    replaceBtn.onclick();
    const result = await p;
    assert.equal(result, 'replace');
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('customAlert and customConflictDialog cover remaining action paths', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = documentStub;
  global.window = {};

  try {
    installDialogHelpers({ getText: (k) => (k === 'okBtn' ? 'OK' : 'Cancel') });

    const alertPromise = window.customAlert('Alert body', 'Alert title');
    assert.equal(map.genericDialogActions.children.length, 1);
    map.genericDialogActions.children[0].onclick();
    await alertPromise;

    const keepPromise = window.customConflictDialog('Conflict', 'Title', 'Replace', 'Keep', 'Cancel');
    map.genericDialogActions.children[1].onclick();
    assert.equal(await keepPromise, 'keep');

    const cancelPromise = window.customConflictDialog('Conflict', 'Title', 'Replace', 'Keep', 'Cancel');
    map.genericDialogActions.children[0].onclick();
    assert.equal(await cancelPromise, 'cancel');
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('customConfirm uses default labels and resolves true on OK', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = documentStub;
  global.window = {};

  try {
    installDialogHelpers({ getText: (k) => (k === 'okBtn' ? 'OK' : 'Cancel') });
    const p = window.customConfirm('Proceed?', 'Confirm');
    assert.equal(map.genericDialogActions.children.length, 2);
    const cancelBtn = map.genericDialogActions.children[0];
    const okBtn = map.genericDialogActions.children[1];
    // Default cancel text length is short, so width override should not be applied.
    assert.equal(cancelBtn.style.width ?? '', '');
    okBtn.onclick();
    assert.equal(await p, true);
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('customPrompt enter key and fallback title/default value branches', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  const prevSetTimeout = global.setTimeout;
  global.document = documentStub;
  global.window = {};
  global.setTimeout = (fn) => {
    fn();
    return 0;
  };

  try {
    installDialogHelpers({ getText: (k) => k });
    const p = window.customPrompt('Message body', '', undefined);
    assert.equal(map.inputDialogTitle.textContent, 'Pulsar');
    assert.equal(map.inputDialogField.value, '');
    map.inputDialogField.value = 'via-enter';
    map.inputDialogField.onkeydown({ key: 'Enter' });
    assert.equal(await p, 'via-enter');
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
    global.setTimeout = prevSetTimeout;
  }
});

test('customConflictDialog fallback labels/title branches', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = documentStub;
  global.window = {};

  try {
    installDialogHelpers({ getText: (k) => k });
    const p = window.customConflictDialog('Conflict body', '', '', '', '');
    assert.equal(map.genericDialogTitle.textContent, 'Conflict');
    assert.equal(map.genericDialogActions.children[0].textContent, 'Cancel');
    assert.equal(map.genericDialogActions.children[1].textContent, 'Keep Both');
    assert.equal(map.genericDialogActions.children[2].textContent, 'Replace');
    map.genericDialogActions.children[2].onclick();
    assert.equal(await p, 'replace');
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});

test('customAlert uses fallback title when empty title is passed', async () => {
  const { map, documentStub } = setupDialogDom();
  const prevDocument = global.document;
  const prevWindow = global.window;
  global.document = documentStub;
  global.window = {};

  try {
    installDialogHelpers({ getText: (k) => k });
    const p = window.customAlert('Notice', '');
    assert.equal(map.genericDialogTitle.textContent, 'Pulsar');
    map.genericDialogActions.children[0].onclick();
    await p;
  } finally {
    global.document = prevDocument;
    global.window = prevWindow;
  }
});
