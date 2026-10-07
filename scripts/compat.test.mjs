import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

test('legacy product commands retain safe errors and exact exit status after forwarding', async () => {
  const backup = await import('./backup.mjs');
  const voices = await import('./qwen-voices.mjs');
  assert.equal(typeof backup.verifyBackup, 'function');
  assert.equal(typeof voices.queryVoice, 'function');
  const health = spawnSync(process.execPath, [fileURLToPath(new URL('./health-check.mjs', import.meta.url)),
    '--project', 'brioche', '--origin', 'http://user:private-test-value@example.test'], { encoding: 'utf8' });
  assert.equal(health.status, 2);
  assert.equal(JSON.parse(health.stdout).reason, 'invalid-options-or-check-failure');
  assert(!`${health.stdout}${health.stderr}`.includes('private-test-value'));
  const seal = spawnSync(process.execPath, [fileURLToPath(new URL('./backup-seal.mjs', import.meta.url)),
    'seal', '--input', 'invalid', '--key-file', 'invalid'], { encoding: 'utf8' });
  assert.equal(seal.status, 1);
  assert.match(seal.stderr, /Backup encryption operation failed/);
});
