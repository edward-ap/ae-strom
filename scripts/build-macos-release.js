// Builds an unsigned, path-sanitized candidate. This script never publishes or signs.
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { publicationEnvironment } from './release-env.js';

if (process.platform !== 'darwin') throw new Error('A macOS host and Apple SDK are required.');
if (process.argv.length !== 2) throw new Error('Use npm run build:mac:release without extra arguments.');
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const env = publicationEnvironment(root, join(root, 'local-data', 'publication-target-macos'), {
  RUSTUP_TOOLCHAIN: '1.96.0',
  MACOSX_DEPLOYMENT_TARGET: '11.0',
});
const result = spawnSync(process.execPath, [
  join(root, 'node_modules/@tauri-apps/cli/tauri.js'), 'build',
  '--target', 'aarch64-apple-darwin', '--bundles', 'app', '--no-sign', '--ci', '--', '--locked',
], { cwd: root, env, stdio: 'inherit' });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
const bundle = join(env.CARGO_TARGET_DIR, 'aarch64-apple-darwin/release/bundle/macos/Æ Strøm.app');
const check = spawnSync(process.execPath, [join(root, 'scripts/check-macos-release.js'), bundle], { cwd: root, stdio: 'inherit' });
if (check.error) throw check.error;
process.exit(check.status ?? 1);
