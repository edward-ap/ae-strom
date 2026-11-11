// Static arm64 bundle check. Does not launch the app, read profiles, or alter signatures.
import { readFileSync, readdirSync, lstatSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assertNoPrivateContent, assertArm64MachO } from './release-guards.js';

if (process.platform !== 'darwin') throw new Error('Run the bundle check on macOS.');
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
if (process.argv.length !== 3) throw new Error('Use: node scripts/check-macos-release.js <Æ Strøm.app>');
const app = resolve(process.argv[2]);
const expected = JSON.parse(readFileSync(join(root, 'tauri.conf.json'), 'utf8'));
const allowed = new Set(['Contents/Info.plist', 'Contents/MacOS/ae-strom', 'Contents/Resources/icon.icns']);
const records = [];
function walk(directory) {
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry), stat = lstatSync(path);
    if (stat.isSymbolicLink()) throw new Error('Unexpected symlink in self-contained candidate bundle.');
    if (stat.isDirectory()) { walk(path); continue; }
    if (!stat.isFile() || !allowed.has(relative(app, path))) throw new Error('Unexpected bundle content; inspect before publication.');
    const bytes = readFileSync(path);
    assertNoPrivateContent(bytes);
    records.push({ path: relative(app, path), bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
  }
}
walk(app);
if (records.length !== allowed.size) throw new Error('Incomplete app bundle.');
const executable = join(app, 'Contents/MacOS/ae-strom');
const bytes = readFileSync(executable);
assertArm64MachO(bytes);
if (!(lstatSync(executable).mode & 0o111)) throw new Error('Executable permissions missing.');
const plistResult = spawnSync('/usr/bin/plutil', ['-convert', 'json', '-o', '-', join(app, 'Contents/Info.plist')], { encoding: 'utf8' });
if (plistResult.status !== 0) throw new Error('Invalid Info.plist.');
const plist = JSON.parse(plistResult.stdout);
if (plist.CFBundleIdentifier !== expected.identifier || plist.CFBundleShortVersionString !== expected.version || plist.CFBundleExecutable !== 'ae-strom' || plist.CFBundlePackageType !== 'APPL') {
  throw new Error('Bundle identity/version differs from source.');
}
const linked = spawnSync('/usr/bin/otool', ['-L', executable], { encoding: 'utf8' });
if (linked.status !== 0) throw new Error('Cannot inspect linked libraries.');
const libraries = linked.stdout.trim().split('\n').slice(1).map(line => line.trim().split(' (')[0]);
if (!libraries.length || libraries.some(name => !name.startsWith('/System/Library/') && !name.startsWith('/usr/lib/'))) {
  throw new Error('Unexpected non-system dynamic library.');
}
console.log(JSON.stringify({ platform: 'macos-arm64', version: expected.version, minimumSystemVersion: plist.LSMinimumSystemVersion,
  personalPathOrCredentialPatterns: 0, linkedLibraries: libraries, files: records,
  distributionStatus: 'local unsigned candidate; not Developer ID signed or notarized' }, null, 2));
