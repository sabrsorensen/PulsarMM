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

test('makeNexusApiCall throws when an authed call has no API key', async () => {
  const api = createNexusApi({
    invoke: async () => ({ status: 200, body: '[]' }),
    getApiKey: () => null,
    cache: new Map(),
  });
  await assert.rejects(api.fetchTrendingMods(), /NexusMods API key required/);
});

test('makeNexusApiCall throws a specific message on 401 and 429', async () => {
  const apiUnauthorized = createNexusApi({
    invoke: async () => ({ status: 401, body: '{}' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiUnauthorized.fetchTrendingMods(), /Invalid NexusMods API key/);

  const apiThrottled = createNexusApi({
    invoke: async () => ({ status: 429, body: '{}' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiThrottled.fetchTrendingMods(), /rate limit exceeded/);
});

test('fetchTrendingMods returns data on success and rethrows on failure', async () => {
  const api = createNexusApi({
    invoke: async () => ({ status: 200, body: JSON.stringify([{ mod_id: 1 }]) }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.deepEqual(await api.fetchTrendingMods(), [{ mod_id: 1 }]);

  const apiFail = createNexusApi({
    invoke: async () => ({ status: 500, body: 'err' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiFail.fetchTrendingMods(), /NexusMods API error: 500/);
});

test('fetchRecentlyUpdatedMods uses default and custom periods, rethrows on failure', async () => {
  const calls = [];
  const api = createNexusApi({
    invoke: async (_cmd, payload) => {
      calls.push(payload.url);
      return { status: 200, body: JSON.stringify([{ mod_id: 2 }]) };
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await api.fetchRecentlyUpdatedMods();
  await api.fetchRecentlyUpdatedMods('1d');
  assert.match(calls[0], /period=1w/);
  assert.match(calls[1], /period=1d/);

  const apiFail = createNexusApi({
    invoke: async () => {
      throw new Error('down');
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiFail.fetchRecentlyUpdatedMods(), /down/);
});

test('fetchModDetails caches successful results and rethrows on failure', async () => {
  let count = 0;
  const cache = new Map();
  const detail = { mod_id: 5, name: 'Test Mod' };
  const api = createNexusApi({
    invoke: async () => {
      count += 1;
      return { status: 200, body: JSON.stringify(detail) };
    },
    getApiKey: () => 'k',
    cache,
  });
  assert.deepEqual(await api.fetchModDetails(5), detail);
  assert.deepEqual(await api.fetchModDetails(5), detail);
  assert.equal(count, 1, 'second call should be served from cache');

  const apiFail = createNexusApi({
    invoke: async () => ({ status: 404, body: '{}' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiFail.fetchModDetails(6));
});

test('fetchModFiles caches successful results, handles missing file lists, and rethrows on failure', async () => {
  let count = 0;
  const cache = new Map();
  const files = { files: [{ file_id: 1 }] };
  const api = createNexusApi({
    invoke: async () => {
      count += 1;
      return { status: 200, body: JSON.stringify(files) };
    },
    getApiKey: () => 'k',
    cache,
  });
  assert.deepEqual(await api.fetchModFiles(7), files);
  assert.deepEqual(await api.fetchModFiles(7), files);
  assert.equal(count, 1, 'second call should be served from cache');

  const apiNoFilesField = createNexusApi({
    invoke: async () => ({ status: 200, body: '{}' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.deepEqual(await apiNoFilesField.fetchModFiles(70), {});

  const apiFail = createNexusApi({
    invoke: async () => {
      throw new Error('nope');
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiFail.fetchModFiles(8));
});

test('fetchModChangelogs returns data on success and an empty object on failure', async () => {
  const api = createNexusApi({
    invoke: async () => ({ status: 200, body: JSON.stringify({ '1.0': ['fix'] }) }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.deepEqual(await api.fetchModChangelogs(9), { '1.0': ['fix'] });

  const apiFail = createNexusApi({
    invoke: async () => ({ status: 500, body: 'err' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  assert.deepEqual(await apiFail.fetchModChangelogs(9), {});
});

test('fetchCompleteModData combines details/files/changelogs and rethrows on failure', async () => {
  const detail = {
    mod_id: 10,
    name: 'Combo',
    summary: 's',
    version: '1.0',
    picture_url: 'p',
    author: 'a',
    updated_timestamp: 1,
    created_timestamp: 0,
    description: 'd',
    unique_downloads: 1,
    endorsement_count: 2,
    status: 'published',
    contains_adult_content: false,
  };
  const files = { files: [{ file_id: 1 }] };
  const changelogs = { '1.0': ['initial'] };

  const api = createNexusApi({
    invoke: async (_cmd, payload) => {
      if (payload.url.endsWith('/mods/10.json')) {
        return { status: 200, body: JSON.stringify(detail) };
      }
      if (payload.url.endsWith('/mods/10/files.json')) {
        return { status: 200, body: JSON.stringify(files) };
      }
      if (payload.url.endsWith('/mods/10/changelogs.json')) {
        return { status: 200, body: JSON.stringify(changelogs) };
      }
      throw new Error(`unexpected url ${payload.url}`);
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });

  const combined = await api.fetchCompleteModData(10);
  assert.equal(combined.mod_id, 10);
  assert.equal(combined.name, 'Combo');
  assert.deepEqual(combined.files, files.files);
  assert.deepEqual(combined.changelogs, changelogs);
  assert.equal(combined.state, 'normal');
  assert.equal(combined.warningMessage, '');

  const apiFail = createNexusApi({
    invoke: async () => ({ status: 500, body: 'err' }),
    getApiKey: () => 'k',
    cache: new Map(),
  });
  await assert.rejects(apiFail.fetchCompleteModData(11));
});

test('fetchCompleteModData falls back to an empty file list and changelog object when absent', async () => {
  const detail = { mod_id: 12, name: 'Sparse' };

  const api = createNexusApi({
    invoke: async (_cmd, payload) => {
      if (payload.url.endsWith('/mods/12.json')) {
        return { status: 200, body: JSON.stringify(detail) };
      }
      if (payload.url.endsWith('/mods/12/files.json')) {
        return { status: 200, body: '{}' };
      }
      if (payload.url.endsWith('/mods/12/changelogs.json')) {
        return { status: 200, body: 'null' };
      }
      throw new Error(`unexpected url ${payload.url}`);
    },
    getApiKey: () => 'k',
    cache: new Map(),
  });

  const combined = await api.fetchCompleteModData(12);
  assert.deepEqual(combined.files, []);
  assert.deepEqual(combined.changelogs, {});
});

