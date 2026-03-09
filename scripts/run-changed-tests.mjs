import { existsSync } from 'node:fs';
import { basename } from 'node:path';
import { spawnSync } from 'node:child_process';

function run(cmd, args, options = {}) {
  const result = spawnSync(cmd, args, {
    stdio: 'pipe',
    encoding: 'utf8',
    ...options,
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || result.stdout || `${cmd} failed`);
  }
  return result.stdout.trim();
}

function runInteractive(cmd, args) {
  const result = spawnSync(cmd, args, { stdio: 'inherit' });
  return result.status ?? 1;
}

const stagedOutput = run('git', ['diff', '--cached', '--name-only', '--diff-filter=ACMR']);
const stagedFiles = stagedOutput
  .split('\n')
  .map((f) => f.trim())
  .filter(Boolean);

if (stagedFiles.length === 0) {
  console.log('No staged files detected. Skipping tests.');
  process.exit(0);
}

const directlyChangedTests = stagedFiles.filter((f) => /^tests\/.+\.test\.js$/.test(f));
const mappedTests = new Set(directlyChangedTests);

let requiresFullSuite = false;
for (const file of stagedFiles) {
  if (file.startsWith('src/') && file.endsWith('.js')) {
    const name = basename(file, '.js');
    const guessedTest = `tests/${name}.test.js`;
    if (existsSync(guessedTest)) {
      mappedTests.add(guessedTest);
    } else {
      requiresFullSuite = true;
    }
    continue;
  }

  if (
    file === 'package.json' ||
    file === 'package-lock.json' ||
    file.startsWith('src-tauri/') ||
    file.startsWith('.github/workflows/')
  ) {
    requiresFullSuite = true;
  }
}

if (requiresFullSuite || mappedTests.size === 0) {
  console.log('Running full test suite (changed files require broad validation)...');
  process.exit(runInteractive('npm', ['test']));
}

const selected = [...mappedTests].sort();
console.log(`Running targeted tests for staged changes:\n- ${selected.join('\n- ')}`);
process.exit(runInteractive('node', ['--test', ...selected]));
