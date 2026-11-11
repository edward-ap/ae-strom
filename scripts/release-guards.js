// Static guards shared by Windows and macOS; no user profile is read.
export function assertNoPrivateContent(bytes) {
  const representations = [bytes.toString('latin1'), bytes.toString('utf16le'), bytes.subarray(1).toString('utf16le')];
  const privatePatterns = [
    /[A-Z]:[\\/]Users[\\/]/i,
    /[A-Z]:[\\/](?:Work|Documents and Settings)[\\/]/i,
    /\/(?:Users|home)\/[a-z0-9_.-]+\//i,
    /(?:gh[pousr]_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,})/,
    /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/,
  ];
  if (privatePatterns.some(pattern => representations.some(value => pattern.test(value)))) {
    throw new Error('Possible personal build path or credential; inspect privately before publication.');
  }
}

export function assertArm64MachO(bytes) {
  if (bytes.length < 32 || bytes.readUInt32LE(0) !== 0xfeedfacf || bytes.readUInt32LE(4) !== 0x0100000c) {
    throw new Error('Expected a thin macOS arm64 Mach-O executable.');
  }
  if (bytes.readUInt32LE(12) !== 2) throw new Error('Expected MH_EXECUTE.');
  const commandCount = bytes.readUInt32LE(16), commandBytes = bytes.readUInt32LE(20);
  if (commandCount === 0 || commandBytes < commandCount * 8 || commandBytes > bytes.length - 32) {
    throw new Error('Invalid Mach-O command bounds.');
  }
  let offset = 32;
  for (let i = 0; i < commandCount; i++) {
    if (offset + 8 > 32 + commandBytes) throw new Error('Truncated Mach-O load command.');
    const size = bytes.readUInt32LE(offset + 4);
    if (size < 8 || offset + size > 32 + commandBytes) throw new Error('Invalid Mach-O load command size.');
    offset += size;
  }
  if (offset !== 32 + commandBytes) throw new Error('Mach-O command size mismatch.');
}
