import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
const phase = process.argv[2];
if (!['before', 'after'].includes(phase)) throw Error('Use before or after');
const folder = `${process.argv[3] || "local-data/ui-polish"}/${phase}`;
mkdirSync(folder, { recursive: true });
const server = spawn(process.execPath, ['scripts/preview.js', '5187'], { stdio: ['ignore', 'pipe', 'inherit'], windowsHide: true });
await new Promise((resolve, reject) => { server.stdout.once('data', resolve); server.once('error', reject); });
let browser;
const records = [];
try {
  browser = await chromium.launch();
  for (const dpr of [1, 1.5]) {
    const scale = dpr === 1 ? '100' : '150';
    const page = await browser.newPage({ viewport: { width: 900, height: 600 }, deviceScaleFactor: dpr, colorScheme: 'light' });
    await page.goto('http://127.0.0.1:5187');
    await page.waitForFunction(() => window.__RADIO_UI_READY__ === true);
    const capture = async name => {
      const clip = await page.locator('.radio-owner').boundingBox();
      await page.screenshot({ path: `${folder}/${name}-${scale}.png`, clip });
      records.push({ name, dpr, rect: clip, buttons: await page.locator('.player-strip button').evaluateAll(elements => elements.map(e => ({ control: e.dataset.control, rect: e.getBoundingClientRect().toJSON(), pressed: e.getAttribute('aria-pressed'), expanded: e.getAttribute('aria-expanded'), icon: e.querySelector('svg')?.outerHTML || null, label: e.getAttribute('aria-label') }))) });
    };
    await capture('normal');
    await page.getByRole('button', { name: 'Station settings', exact: true }).click();
    await capture('settings');
    await page.getByRole('button', { name: 'Equalizer', exact: true }).click();
    await capture('equalizer');
    await page.getByRole('button', { name: 'Mute', exact: true }).click();
    await capture('muted');
    await page.getByRole('button', { name: 'Play', exact: true }).click();
    await page.getByRole('button', { name: 'Collapse player', exact: true }).click();
    await capture('compact');
    await page.close();
  }
  writeFileSync(`${folder}/metrics.json`, JSON.stringify({ phase, syntheticOnly: true, realOSInput: false, records }, null, 2));
} finally { await browser?.close(); server.kill(); }
console.log(`Captured ${phase}: 5 views at browser DPR 1/1.5, current app theme; no audio.`);
