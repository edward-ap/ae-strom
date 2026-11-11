// Static publication guard. This does not start the application or inspect a user profile.
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { assertNoPrivateContent } from './release-guards.js';
const file = process.argv[2];
if (!file) throw new Error('Use: node scripts/check-release.js <Windows x64 EXE>');
const bytes = readFileSync(file);
const fail = message => { throw new Error(message); };
if (bytes.toString('ascii', 0, 2) !== 'MZ') fail('Not a PE executable.');
const pe = bytes.readUInt32LE(0x3c);
if (bytes.toString('ascii', pe, pe + 4) !== 'PE\0\0' || bytes.readUInt16LE(pe + 4) !== 0x8664) fail('Expected Windows x64 PE.');
assertNoPrivateContent(bytes);
const optional = pe + 24;
if (bytes.readUInt16LE(optional) !== 0x20b) fail('Expected PE32+.');
const sections = optional + bytes.readUInt16LE(pe + 20);
const sectionCount = bytes.readUInt16LE(pe + 6);
function offset(rva) {
  for (let i = 0; i < sectionCount; i++) {
    const s = sections + i * 40;
    const address = bytes.readUInt32LE(s + 12);
    const length = Math.max(bytes.readUInt32LE(s + 8), bytes.readUInt32LE(s + 16));
    if (rva >= address && rva < address + length) return bytes.readUInt32LE(s + 20) + rva - address;
  }
  fail('Invalid PE debug RVA.');
}
const debugRva = bytes.readUInt32LE(optional + 112 + 6 * 8);
const debugSize = bytes.readUInt32LE(optional + 116 + 6 * 8);
let codeViewRecords = 0;
if (debugRva) {
  const start = offset(debugRva);
  for (let i = 0; i + 28 <= debugSize; i += 28) {
    const entry = start + i;
    if (bytes.readUInt32LE(entry + 12) !== 2) continue;
    const size = bytes.readUInt32LE(entry + 16);
    const raw = bytes.readUInt32LE(entry + 24);
    if (size >= 25 && bytes.toString('ascii', raw, raw + 4) === 'RSDS') {
      const end = bytes.indexOf(0, raw + 24);
      if (end < raw + 24 || end >= raw + size) fail('Malformed CodeView record.');
      if (/[\\/]/.test(bytes.toString('utf8', raw + 24, end))) fail('CodeView records an absolute or relative PDB path.');
      codeViewRecords++;
    }
  }
}
console.log(JSON.stringify({ platform: 'windows-x86_64', bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex'), personalPathOrCredentialPatterns: 0, codeViewRecords }, null, 2));
