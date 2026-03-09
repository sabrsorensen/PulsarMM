import test from 'node:test';
import assert from 'node:assert/strict';

import {
  getBaseName,
  isNewerVersionAvailable,
} from '../src/utils/versionUtils.js';

test('getBaseName normalizes archive/file style names', () => {
  assert.equal(getBaseName('Cool-Mod_v1.2.3.zip'), 'coolmod');
  assert.equal(getBaseName('Another Mod-123-456.7z'), 'anothermod');
  assert.equal(getBaseName('My_Mod_v2a.pak'), 'mymod');
});

test('getBaseName handles empty values', () => {
  assert.equal(getBaseName(''), '');
  assert.equal(getBaseName(null), '');
});

test('isNewerVersionAvailable compares numeric parts', () => {
  assert.equal(isNewerVersionAvailable('1.2.3', '1.2.4'), true);
  assert.equal(isNewerVersionAvailable('1.2.3', '1.2.3'), false);
  assert.equal(isNewerVersionAvailable('1.2.3', '1.2.2'), false);
});

test('isNewerVersionAvailable handles missing inputs', () => {
  assert.equal(isNewerVersionAvailable('', '1.0.0'), false);
  assert.equal(isNewerVersionAvailable('1.0.0', ''), false);
});

test('isNewerVersionAvailable handles suffix semantics', () => {
  assert.equal(isNewerVersionAvailable('1.0', '1.0-beta'), true);
  assert.equal(isNewerVersionAvailable('1.0-beta', '1.0'), false);
  assert.equal(isNewerVersionAvailable('1.0-alpha', '1.0-beta'), true);
});

test('isNewerVersionAvailable handles mixed length numeric versions', () => {
  assert.equal(isNewerVersionAvailable('1.2', '1.2.1'), true);
  assert.equal(isNewerVersionAvailable('1.2.1', '1.2'), false);
});

test('isNewerVersionAvailable handles non-numeric version strings', () => {
  assert.equal(isNewerVersionAvailable('beta', 'alpha'), false);
  assert.equal(isNewerVersionAvailable('alpha', 'beta'), false);
});
