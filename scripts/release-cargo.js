// Publication builds remap machine-specific diagnostic/source paths; app code is unchanged.
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { publicationEnvironment } from './release-env.js';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
if (!['build', 'test', 'clippy'].includes(args[0])) {
  throw new Error('Use: node scripts/release-cargo.js <build|test|clippy> <Cargo arguments>');
}
const result = spawnSync('cargo', args, {
  cwd: root,
  env: publicationEnvironment(root, join(root, 'local-data', 'publication-target')),
  stdio: 'inherit',
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
