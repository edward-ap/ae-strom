// Shared path sanitization for isolated publication builds; no runtime behavior changes.
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import { join } from 'node:path';

export function publicationEnvironment(root, targetDirectory, extra = {}) {
  if (process.env.RUSTFLAGS || process.env.CARGO_ENCODED_RUSTFLAGS || process.env.RUSTDOCFLAGS) {
    throw new Error('Use a clean Rust flags environment for the reviewed publication build.');
  }
  const env = { ...process.env, ...extra };
  const compiler = spawnSync('rustc', ['--print', 'sysroot'], { encoding: 'utf8', env });
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
  if (process.platform === 'win32') flags.push('-C', 'link-arg=/PDBALTPATH:ae-strom.pdb');
  return {
    ...env,
    CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR || targetDirectory,
    CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f'),
  };
}
