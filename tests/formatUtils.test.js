import test from 'node:test';
import assert from 'node:assert/strict';

import {
  formatBytes,
  formatDate,
  formatNexusDate,
  mapLangCode,
} from '../src/utils/formatUtils.js';

test('formatBytes handles zero and small values', () => {
  assert.equal(formatBytes(0), '0 B');
  assert.equal(formatBytes(1), '1 B');
  assert.equal(formatBytes(1024), '1 KB');
});

test('formatBytes respects precision argument', () => {
  assert.equal(formatBytes(1536, 1), '1.5 KB');
  assert.equal(formatBytes(1536, 0), '2 KB');
  assert.equal(formatBytes(1536, -1), '2 KB');
});

test('formatDate returns placeholder for falsy timestamp', () => {
  assert.equal(formatDate(0), '...');
  assert.equal(formatDate(null), '...');
});

test('formatDate formats unix seconds using local date format', () => {
  const ts = 1710000000;
  assert.equal(formatDate(ts), new Date(ts * 1000).toLocaleDateString());
});

test('formatNexusDate returns placeholder for falsy timestamp', () => {
  assert.equal(formatNexusDate(0, 'en-US'), '---');
});

test('formatNexusDate uses requested locale with day/month/year', () => {
  const ts = 1710000000;
  const expected = new Date(ts * 1000).toLocaleDateString('en-US', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  });
  assert.equal(formatNexusDate(ts, 'en-US'), expected);
});

test('mapLangCode remaps special language codes and passes through others', () => {
  assert.equal(mapLangCode('cn'), 'zh-CN');
  assert.equal(mapLangCode('kr'), 'ko-KR');
  assert.equal(mapLangCode('pt'), 'pt');
});
