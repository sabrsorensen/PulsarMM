import test from 'node:test';
import assert from 'node:assert/strict';

import { CURATED_LIST_URL, CACHE_DURATION_MS } from '../src/config/appConstants.js';

test('app constants are defined as expected', () => {
  assert.match(CURATED_LIST_URL, /curated_list\.json$/);
  assert.equal(CACHE_DURATION_MS, 60 * 60 * 1000);
});

