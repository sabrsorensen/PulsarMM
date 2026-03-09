import test from 'node:test';
import assert from 'node:assert/strict';

import { resolveUpdateCandidate } from '../src/features/updateResolver.js';
import { getBaseName, isNewerVersionAvailable } from '../src/utils/versionUtils.js';

const helpers = { getBaseName, isNewerVersionAvailable };

test('returns null when local mod info is missing', () => {
  assert.equal(resolveUpdateCandidate('mod', null, [], helpers), null);
});

test('returns null when required installed metadata is missing', () => {
  const cached = { local_info: { mod_id: '', version: '' } };
  assert.equal(resolveUpdateCandidate('mod', cached, [], helpers), null);
});

test('returns null when version is missing even if mod id exists', () => {
  const cached = { local_info: { mod_id: '10', version: '', file_id: '1' } };
  assert.equal(resolveUpdateCandidate('mod', cached, [], helpers), null);
});

test('returns null when remote mod data is missing', () => {
  const cached = { local_info: { mod_id: '10', version: '1.0', file_id: '1' } };
  assert.equal(resolveUpdateCandidate('mod', cached, [], helpers), null);
});

test('returns null when remote mod files are not an array', () => {
  const cached = { local_info: { mod_id: '10', version: '1.0', file_id: '1' } };
  const curatedData = [{ mod_id: 10, files: null }];
  assert.equal(resolveUpdateCandidate('mod', cached, curatedData, helpers), null);
});

test('picks latest matching candidate and returns update payload', () => {
  const cached = {
    local_info: {
      mod_id: '10',
      version: '1.0.0',
      file_id: '100',
      install_source: 'Cool Mod v1.0.0.zip',
    },
  };

  const curatedData = [
    {
      mod_id: 10,
      name: 'Cool Mod',
      files: [
        { file_id: 100, name: 'Cool Mod v1.0.0', file_name: 'coolmod-100.zip', version: '1.0.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 101, name: 'Cool Mod v1.1.0', file_name: 'coolmod-101.zip', version: '1.1.0', uploaded_timestamp: 3, category_name: 'MAIN' },
        { file_id: 102, name: 'Cool Mod v1.0.5', file_name: 'coolmod-102.zip', version: '1.0.5', uploaded_timestamp: 2, category_name: 'OLD_VERSION' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('COOL_MOD', cached, curatedData, helpers);

  assert.ok(result);
  assert.equal(result.modId, '10');
  assert.equal(result.installedVersion, '1.0.0');
  assert.equal(result.latestVersion, '1.1.0');
  assert.equal(result.latestFileId, '101');
  assert.equal(result.modName, 'Cool Mod');
  assert.match(result.nexusUrl, /nomanssky\/mods\/10$/);
});

test('falls back to install_source when installed file id is not in remote files', () => {
  const cached = {
    local_info: {
      mod_id: '11',
      version: '1.0',
      file_id: '999',
      install_source: 'Some Mod v1.0.zip',
    },
  };

  const curatedData = [
    {
      mod_id: 11,
      name: 'Some Mod',
      files: [
        { file_id: 200, name: 'Some Mod v1.2', file_name: 'somemod-200.zip', version: '1.2', uploaded_timestamp: 10, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('SOMEMOD', cached, curatedData, helpers);
  assert.ok(result);
  assert.equal(result.latestVersion, '1.2');
});

test('falls back to mod folder name when install_source and file match are unavailable', () => {
  const cached = {
    local_info: {
      mod_id: '12',
      version: '1.0',
      file_id: '999',
      install_source: '',
    },
  };

  const curatedData = [
    {
      mod_id: 12,
      name: 'Folder Match',
      files: [
        { file_id: 300, name: 'FOLDER_MATCH v1.1', file_name: 'folder_match_300.zip', version: '1.1', uploaded_timestamp: 5, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('folder-match', cached, curatedData, helpers);
  assert.ok(result);
  assert.equal(result.latestFileId, '300');
});

test('returns null when no newer version is available', () => {
  const cached = {
    local_info: { mod_id: '13', version: '2.0', file_id: '401', install_source: 'Stable v2.0.zip' },
  };
  const curatedData = [
    {
      mod_id: 13,
      name: 'Stable',
      files: [
        { file_id: 401, name: 'Stable v2.0', file_name: 'stable-401.zip', version: '2.0', uploaded_timestamp: 100, category_name: 'MAIN' },
      ],
    },
  ];

  assert.equal(resolveUpdateCandidate('stable', cached, curatedData, helpers), null);
});

test('returns null when installed base name cannot be resolved', () => {
  const cached = {
    local_info: {
      mod_id: '14',
      version: '1.0',
      file_id: '999',
      install_source: '',
    },
  };
  const curatedData = [{ mod_id: 14, name: 'No Base', files: [{ file_id: 1, name: 'No Base v1.1', version: '1.1', uploaded_timestamp: 1, category_name: 'MAIN' }] }];
  assert.equal(resolveUpdateCandidate('', cached, curatedData, helpers), null);
});

test('returns null when all candidate files are filtered out', () => {
  const cached = {
    local_info: {
      mod_id: '15',
      version: '1.0',
      file_id: '10',
      install_source: 'Filter Mod v1.0.zip',
    },
  };
  const curatedData = [
    {
      mod_id: 15,
      name: 'Filter Mod',
      files: [
        { file_id: 10, name: 'Filter Mod v1.0', file_name: 'filter-mod-10.zip', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 11, name: 'Filter Mod v1.2', file_name: 'filter-mod-11.zip', version: '1.2', uploaded_timestamp: 2, category_name: 'OLD_VERSION' },
        { file_id: 12, name: 'Different Mod v2.0', file_name: 'different-12.zip', version: '2.0', uploaded_timestamp: 3, category_name: 'MAIN' },
      ],
    },
  ];

  assert.equal(resolveUpdateCandidate('filter-mod', cached, curatedData, helpers), null);
});

test('returns null when candidate list is truly empty after matching', () => {
  const cached = {
    local_info: {
      mod_id: '24',
      version: '1.0',
      file_id: '9999',
      install_source: 'NoMatch Mod v1.0.zip',
    },
  };
  const curatedData = [
    {
      mod_id: 24,
      name: 'NoMatch Mod',
      files: [
        { file_id: 1000, name: 'Completely Different v1.0', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 1001, name: 'Another Different v1.1', version: '1.1', uploaded_timestamp: 2, category_name: 'MAIN' },
      ],
    },
  ];
  assert.equal(resolveUpdateCandidate('nomatch-mod', cached, curatedData, helpers), null);
});

test('builds payload with fallback installedFileId and latest file/display names', () => {
  const cached = {
    local_info: {
      mod_id: '16',
      version: '1.0',
      file_id: null,
      install_source: 'Payload Mod v1.0.zip',
    },
  };
  const curatedData = [
    {
      mod_id: 16,
      name: 'Payload Mod',
      files: [
        { file_id: 20, name: 'Payload Mod v1.2', file_name: 'payload-mod-20.zip', version: '1.2', uploaded_timestamp: 5, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('payload-mod', cached, curatedData, helpers);
  assert.ok(result);
  assert.equal(result.installedFileId, '');
  assert.equal(result.latestFileName, 'payload-mod-20.zip');
  assert.equal(result.latestDisplayName, 'Payload Mod v1.2');
});

test('uses file_name when name is missing and falls back modName to folder name', () => {
  const cached = {
    local_info: {
      mod_id: '17',
      version: '1.0',
      file_id: '501',
      install_source: 'fallback-file-name-v1.0.zip',
    },
  };
  const curatedData = [
    {
      mod_id: 17,
      name: '',
      files: [
        { file_id: 501, file_name: 'fallback-file-name-v1.0.zip', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 502, file_name: 'fallback-file-name-v1.2.zip', version: '1.2', uploaded_timestamp: 2, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('folder-fallback', cached, curatedData, helpers);
  assert.ok(result);
  assert.equal(result.latestFileName, 'fallback-file-name-v1.2.zip');
  assert.equal(result.latestDisplayName, 'fallback-file-name-v1.2.zip');
  assert.equal(result.modName, 'folder-fallback');
});

test('uses newest candidate name when file_name missing and keeps remote mod name', () => {
  const cached = {
    local_info: {
      mod_id: '18',
      version: '1.0',
      file_id: '601',
      install_source: 'Name-Only Mod v1.0.zip',
    },
  };
  const curatedData = [
    {
      mod_id: 18,
      name: 'Name Only Mod',
      files: [
        { file_id: 601, name: 'Name-Only Mod v1.0', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 602, name: 'Name-Only Mod v1.3', version: '1.3', uploaded_timestamp: 5, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('name-only-mod', cached, curatedData, helpers);
  assert.ok(result);
  assert.equal(result.latestFileName, 'Name-Only Mod v1.3');
  assert.equal(result.latestDisplayName, 'Name-Only Mod v1.3');
  assert.equal(result.modName, 'Name Only Mod');
});

test('helper-mocked path covers latestVersion fallback and displayName fallback to remote name', () => {
  const mockedHelpers = {
    getBaseName: () => 'same',
    isNewerVersionAvailable: () => true,
  };
  const cached = {
    local_info: {
      mod_id: '21',
      version: '1.0',
      file_id: '701',
      install_source: 'anything',
    },
  };
  const curatedData = [
    {
      mod_id: 21,
      name: 'Remote Name',
      files: [
        { file_id: 701, name: 'Installed Name', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 702, version: '', uploaded_timestamp: 2, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('folder-name', cached, curatedData, mockedHelpers);
  assert.ok(result);
  assert.equal(result.latestVersion, '');
  assert.equal(result.latestFileName, '');
  assert.equal(result.latestDisplayName, 'Remote Name');
});

test('helper-mocked path covers displayName fallback to folder when remote name missing', () => {
  const mockedHelpers = {
    getBaseName: () => 'same',
    isNewerVersionAvailable: () => true,
  };
  const cached = {
    local_info: {
      mod_id: '22',
      version: '1.0',
      file_id: '801',
      install_source: '',
    },
  };
  const curatedData = [
    {
      mod_id: 22,
      name: '',
      files: [
        { file_id: 801, name: 'Installed Name', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 802, uploaded_timestamp: 2, category_name: 'MAIN' },
      ],
    },
  ];

  const result = resolveUpdateCandidate('folder-fallback-name', cached, curatedData, mockedHelpers);
  assert.ok(result);
  assert.equal(result.latestDisplayName, 'folder-fallback-name');
  assert.equal(result.modName, 'folder-fallback-name');
});

test('installed nexus file base-name uses fallback to file_name when name is missing', () => {
  const calls = [];
  const mockedHelpers = {
    getBaseName: (value) => {
      calls.push(value);
      return 'same';
    },
    isNewerVersionAvailable: () => true,
  };
  const cached = {
    local_info: {
      mod_id: '23',
      version: '1.0',
      file_id: '901',
      install_source: '',
    },
  };
  const curatedData = [
    {
      mod_id: 23,
      name: 'Fallback Installed',
      files: [
        { file_id: 901, file_name: 'installed-file-name.zip', version: '1.0', uploaded_timestamp: 1, category_name: 'MAIN' },
        { file_id: 902, name: 'update-name', version: '1.2', uploaded_timestamp: 2, category_name: 'MAIN' },
      ],
    },
  ];
  const result = resolveUpdateCandidate('unused', cached, curatedData, mockedHelpers);
  assert.ok(result);
  assert.ok(calls.includes('installed-file-name.zip'));
});
