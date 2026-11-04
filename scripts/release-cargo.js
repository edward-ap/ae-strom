// Publication builds remap machine-specific diagnostic/source paths; app code is unchanged.
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
if (!['build', 'test', 'clippy'].includes(args[0])) {
  throw new Error('Use: node scripts/release-cargo.js <build|test|clippy> <Cargo arguments>');
}
if (process.env.RUSTFLAGS || process.env.CARGO_ENCODED_RUSTFLAGS || process.env.RUSTDOCFLAGS) {
  throw new Error('Use a clean Rust flags environment for the reviewed publication build.');
}
const compiler = spawnSync('rustc', ['--print', 'sysroot'], { encoding: 'utf8' });
if (compiler.status !== 0) throw new Error('rustc sysroot lookup failed.');
const flags = [];
const mappings = [
  [homedir(), '/build/user'],
  [root, '/build/ae-strom'],
  [process.env.CARGO_HOME || join(homedir(), '.cargo'), '/cargo'],
  [compiler.stdout.trim(), '/rustc/toolchain'],
];
for (const [source, destination] of mappings) {
  for (const prefix of new Set([source, source.replaceAll('\\', '/')])) {
    flags.push(`--remap-path-prefix=${prefix}=${destination}`);
  }
}
// MSVC may independently record its PDB location; never ship an absolute host path.
if (process.platform === 'win32') flags.push('-C', 'link-arg=/PDBALTPATH:ae-strom.pdb');
const result = spawnSync('cargo', args, {
  cwd: root,
  env: {
    ...process.env,
    CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR || join(root, 'local-data', 'publication-target'),
    CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f'),
  },
  stdio: 'inherit',
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
