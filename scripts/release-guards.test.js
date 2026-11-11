import test from 'node:test';
import assert from 'node:assert/strict';
import { assertNoPrivateContent, assertArm64MachO } from './release-guards.js';

test('publication guard accepts remapped paths', () => {
  assert.doesNotThrow(() => assertNoPrivateContent(Buffer.from('/build/user /build/ae-strom/src/main.rs /cargo/registry /rustc/toolchain')));
});
for (const path of ['/Users/example/project/src/main.rs', '/home/example/repo/file', 'C:\\Users\\example\\file', 'D:/Work/project/src/main.rs']) {
  test(`publication guard rejects personal paths in all encodings: ${path}`, () => {
    for (const encoding of ['utf8', 'utf16le']) {
      const bytes = Buffer.from(path, encoding);
      assert.throws(() => assertNoPrivateContent(bytes), /Possible personal/);
      assert.throws(() => assertNoPrivateContent(Buffer.concat([Buffer.from([1]), bytes])), /Possible personal/);
    }
  });
}
test('publication guard rejects credential-shaped data', () => {
  for (const text of ['ghp_' + 'a'.repeat(32), 'github_pat_' + 'a'.repeat(32), '-----BEGIN PRIVATE KEY-----']) {
    assert.throws(() => assertNoPrivateContent(Buffer.from(text)), /Possible personal/);
  }
});
function macho() {
  const bytes = Buffer.alloc(40);
  bytes.writeUInt32LE(0xfeedfacf, 0); bytes.writeUInt32LE(0x0100000c, 4); bytes.writeUInt32LE(2, 12);
  bytes.writeUInt32LE(1, 16); bytes.writeUInt32LE(8, 20); bytes.writeUInt32LE(8, 36);
  return bytes;
}
test('Mach-O guard accepts arm64 executable header', () => assert.doesNotThrow(() => assertArm64MachO(macho())));
test('Mach-O guard rejects x64, fat, truncated and malformed binaries', () => {
  for (const length of [0, 4, 31, 36]) assert.throws(() => assertArm64MachO(macho().subarray(0, length)));
  const x64 = macho(); x64.writeUInt32LE(0x01000007, 4); assert.throws(() => assertArm64MachO(x64));
  const fat = macho(); fat.writeUInt32BE(0xcafebabe, 0); assert.throws(() => assertArm64MachO(fat));
  const wrongSize = macho(); wrongSize.writeUInt32LE(16, 36); assert.throws(() => assertArm64MachO(wrongSize));
  const library = macho(); library.writeUInt32LE(6, 12); assert.throws(() => assertArm64MachO(library));
});
