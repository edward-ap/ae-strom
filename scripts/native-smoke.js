// Programmatic native UI/IPC checks with fresh fake profiles, never an audio device.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const executable = resolve(process.argv[2] || join(root, 'target/release', process.platform === 'win32' ? 'ae-strom.exe' : 'ae-strom'));
const output = join(root, 'local-data', `native-matrix-${Date.now()}`);
mkdirSync(output, { recursive: true });
const results = [];
function run(name, config, reason, upward = false) {
  const data = join(output, name);
  mkdirSync(data);
  if (config) writeFileSync(join(data, 'config.json'), JSON.stringify(config));
  const child = spawnSync(executable, ['--fake', '--smoke', '--data-dir', data], { cwd: root, encoding: 'utf8', timeout: 60000 });
  writeFileSync(join(data, 'stdout.log'), child.stdout || '');
  writeFileSync(join(data, 'stderr.log'), child.stderr || '');
  assert.equal(child.error, undefined, `Native process failed: ${child.error?.message}; evidence ${data}`);
  assert.equal(child.status, 0, `Native process exit ${child.status}: ${child.stderr}`);
  const result = JSON.parse(readFileSync(join(data, 'native-ui.json'), 'utf8'));
  assert.equal(result.ui.pass, true, JSON.stringify(result.ui.checks.filter(check => !check.pass)));
  const host = result.host, startup = host.startup;
  assert.equal(host.mode, 'fake');
  assert.equal(host.audioAllowed, false);
  assert.equal(host.configFile, join(data, 'config.json'));
  assert.equal(host.identifier, 'local.edwardap.ae-strom');
  assert.equal(host.windowTitle, 'Æ Strøm');
  assert.equal(startup.visibleBefore, false);
  assert.equal(startup.visibleAfter, true);
  assert.equal(startup.ui.fontLoaded, true);
  assert.equal(startup.planned.reason, reason);
  assert.equal(startup.arbitraryDelay, false);
  const close = (a, b) => Math.abs(a - b) <= 1; // native point/backing pixel rounding only
  for (const axis of ['x', 'y']) {
    assert(close(startup.positionBeforeShow[axis], startup.planned[axis]), `${name}: ${axis} not positioned before show`);
    assert(close(startup.positionAfterShow[axis], startup.positionBeforeShow[axis]), `${name}: ${axis} moved on show`);
  }
  assert.deepEqual(startup.sizeBeforeShow, startup.sizeAfterShow);
  if (process.platform === 'darwin') {
    assert.equal(host.webviewStorage, 'nonpersistent');
    assert.equal(host.webviewDirectory, null);
  }
  let compact = false, expanded = false;
  for (const layout of host.layouts) {
    const before = layout.outerBefore, after = layout.outerAfter;
    assert(close(layout.stripYBefore, layout.stripYAfter), `${name}: player strip moved`);
    assert.equal(layout.actualPhysical.width, Math.round(layout.requestedLogical[0] * layout.scaleFactor));
    assert.equal(layout.actualPhysical.height, Math.round(layout.requestedLogical[1] * layout.scaleFactor));
    if (layout.horizontalAnchor === 'right') assert(close(before.position.x + before.size.width, after.position.x + after.size.width), `${name}: right edge moved`);
    if (layout.horizontalAnchor === 'left') assert(close(before.position.x, after.position.x), `${name}: left edge moved`);
    for (const [axis, dimension] of [['x', 'width'], ['y', 'height']]) {
      const start = layout.workArea.position[axis], size = layout.workArea.size[dimension];
      if (after.size[dimension] <= size) {
        assert(after.position[axis] >= start - 1 && after.position[axis] + after.size[dimension] <= start + size + 1, `${name}: window exceeds work area`);
      }
    }
    compact ||= layout.compact && before.size.width > after.size.width;
    expanded ||= !layout.compact && before.size.width < after.size.width;
  }
  assert(compact && expanded, `${name}: missing compact/expand transition`);
  if (upward) assert(host.layouts.some(layout => layout.panelsAbove), `${name}: no upward panel`);
  const row = { name, pass: true, checks: result.ui.checks.length, reason, scaleFactor: host.scaleFactor };
  results.push(row);
  console.log(JSON.stringify(row));
  return result;
}
const first = run('absent', null, 'centered-default');
const template = JSON.parse(readFileSync(join(output, 'absent/config.json'), 'utf8'));
const m = first.host.startup.monitors.find(m => m.primary) || first.host.startup.monitors[0];
const fixture = (x, y) => ({ ...structuredClone(template), windowPosValid: true, windowX: Math.round(x), windowY: Math.round(y) });
run('restored', fixture(m.x + 100, m.y + 100), 'restored');
run('offscreen', fixture(1000000, 1000000), 'centered-offscreen');
run('clamped', fixture(m.x + m.width - 20, m.y + 100), 'clamped');
run('bottom', fixture(m.x + 100, m.y + m.height - 36 * m.scale), 'restored', true);
for (const [index, monitor] of first.host.startup.monitors.entries()) {
  if (!monitor.primary) run(`monitor-${index}`, fixture(monitor.x + 100, monitor.y + 100), 'restored');
}
writeFileSync(join(output, 'summary.json'), JSON.stringify({ pass: true, executable, results, audio: false, physicalInput: false, firstCompositorFrameObserved: false }, null, 2));
console.log(`PASS: native fake matrix; evidence ${output}`);
