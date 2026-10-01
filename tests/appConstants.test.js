import test from 'node:test';
import assert from 'node:assert/strict';

import { CACHE_DURATION_MS } from '../src/config/appConstants.js';

test('app constants are defined as expected', () => {
  assert.equal(CACHE_DURATION_MS, 60 * 60 * 1000);
});

