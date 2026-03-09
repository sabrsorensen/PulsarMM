import test from 'node:test';
import assert from 'node:assert/strict';

import { createNexusApi } from '../src/features/nexusApi.js';

test('fetchDownloadUrlFromNexus returns URI on success', async () => {
  const calls = [];
  const api = createNexusApi({
    invoke: async (cmd, payload) => {
      calls.push({ cmd, payload });
      return { status: 200, body: JSON.stringify([{ URI: 'https://example.com/file.zip' }]) };
    },
    getApiKey: () => 'k123',
    cache: new Map(),
  });

  const url = await api.fetchDownloadUrlFromNexus(42, 99, 'foo=bar');
  assert.equal(url, 'https://example.com/file.zip');
  assert.equal(calls.length, 1);
  assert.equal(calls[0].cmd, 'http_request');
  assert.match(calls[0].payload.url, /mods\/42\/files\/99\/download_link\.json\?foo=bar$/);
  assert.equal(calls[0].payload.headers.apikey, 'k123');
});

test('fetchDownloadUrlFromNexus returns null on non-2xx and parse issues', async () => {
  const apiA = createNexusApi({
    invoke: async () => ({ status: 500, body: 'bad' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.equal(await apiA.fetchDownloadUrlFromNexus(1, 2), null);

  const apiB = createNexusApi({
    invoke: async () => {
      throw new Error('network down');
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.equal(await apiB.fetchDownloadUrlFromNexus(1, 2), null);
});

test('fetchModFilesFromNexus uses cache and stores successful responses', async () => {
  let count = 0;
  const cache = new Map();
  const data = { files: [{ file_id: 1 }] };
  const api = createNexusApi({
    invoke: async () => {
      count += 1;
      return { status: 200, body: JSON.stringify(data) };
    },
    getApiKey: () => 'k',
    cache,
  });

  const first = await api.fetchModFilesFromNexus(123);
  const second = await api.fetchModFilesFromNexus('123');

  assert.deepEqual(first, data);
  assert.deepEqual(second, data);
  assert.equal(count, 1);
  assert.deepEqual(cache.get('123'), data);
});

test('fetchModFilesFromNexus returns null on request failures', async () => {
  const apiA = createNexusApi({
    invoke: async () => ({ status: 404, body: '{}' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.equal(await apiA.fetchModFilesFromNexus(3), null);

  const apiB = createNexusApi({
    invoke: async () => {
      throw new Error('oops');
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.equal(await apiB.fetchModFilesFromNexus(3), null);
});

