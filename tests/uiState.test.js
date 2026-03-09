import test from 'node:test';
import assert from 'node:assert/strict';

import {
  DEFAULT_WIDTH,
  PANEL_OPEN_WIDTH,
  SCROLL_SPEED,
  createAppState,
  createDragState,
  createScrollState,
  createDownloadSortState,
} from '../src/state/uiState.js';

test('ui constants remain stable', () => {
  assert.equal(DEFAULT_WIDTH, 950);
  assert.equal(PANEL_OPEN_WIDTH, 1300);
  assert.equal(SCROLL_SPEED, 5);
});

test('createAppState returns expected defaults', () => {
  const state = createAppState();
  assert.equal(state.activeProfile, 'Default');
  assert.equal(state.modsPerPage, 20);
  assert.equal(state.currentPage, 1);
  assert.ok(state.installedModsMap instanceof Map);
  assert.ok(state.modDataCache instanceof Map);
  assert.ok(state.selectedModNames instanceof Set);
  assert.ok(state.selectedDownloadIds instanceof Set);
});

test('createAppState returns isolated mutable collections per call', () => {
  const a = createAppState();
  const b = createAppState();

  a.installedModsMap.set('x', 'y');
  a.selectedModNames.add('mod');

  assert.equal(b.installedModsMap.size, 0);
  assert.equal(b.selectedModNames.size, 0);
});

test('createDragState and createScrollState defaults are stable', () => {
  const drag = createDragState();
  assert.equal(drag.draggedElement, null);
  assert.equal(drag.offsetX, 0);
  assert.equal(drag.offsetY, 0);

  const scroll = createScrollState();
  assert.equal(scroll.isScrollingUp, false);
  assert.equal(scroll.isScrollingDown, false);
  assert.equal(scroll.animationFrameId, null);
});

test('createDownloadSortState defaults to date desc', () => {
  const sort = createDownloadSortState();
  assert.deepEqual(sort, { key: 'date', direction: 'desc' });
});

