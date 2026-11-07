import { test, expect } from '@playwright/test';

for (const [method, synchronous] of [['subscribeState', true], ['subscribeSpectrum', false], ['getSnapshot', false], ['setWindowLayout', false]]) {
  test(`native startup rejects ${method} failure before announcing readiness`, async ({ page }) => {
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(() => { window.__TAURI__ = {}; });
    await page.route('**/bridge.js', route => route.fulfill({ contentType: 'text/javascript', body: `
      import { createPreviewBridge } from './preview-bridge.js';
      export function createTauriBridge() {
        const bridge = createPreviewBridge();
        window.bootProbe = { ready: 0, shutdown: 0 };
        bridge.uiReady = async () => { window.bootProbe.ready++; };
        bridge.shutdown = async () => { window.bootProbe.shutdown++; bridge.destroy(); };
        bridge.${method} = ${synchronous ? '' : 'async'} () => { throw new Error('Injected ${method} failure'); };
        return bridge;
      }
    ` }));
    await page.goto('/');
    await page.waitForFunction(() => window.__RADIO_BOOT_ERROR__);
    expect(await page.evaluate(() => ({ error: window.__RADIO_BOOT_ERROR__, ...window.bootProbe, readyFlag: window.__AE_STROM_READY__ === true }))).toEqual({
      error: `Error: Injected ${method} failure`, ready: 0, shutdown: 1, readyFlag: false,
    });
    await expect(page.locator('.radio-owner')).toHaveCount(0);
    expect(errors).toEqual([]);
  });
}

test('failed startup cleans a subscription that resolves after owner teardown', async ({ page }) => {
  await page.goto('/');
  await page.waitForFunction(() => window.__RADIO_UI_READY__ === true);
  const result = await page.evaluate(async () => {
    window.radioPreview.ui.destroy(); window.radioPreview.bridge.destroy();
    const { createRadioUI } = await import('/radio.js');
    const { createPreviewBridge } = await import('/preview-bridge.js');
    const bridge = createPreviewBridge();
    let finishSubscription, disposed = 0, snapshots = 0;
    bridge.subscribeState = async () => { throw new Error('subscription failed'); };
    bridge.subscribeSpectrum = () => new Promise(resolve => { finishSubscription = () => resolve(() => disposed++); });
    bridge.getSnapshot = async () => { snapshots++; };
    const ui = createRadioUI(document.querySelector('#radio'), bridge);
    const error = await ui.ready.then(() => 'unexpected success', error => error.message);
    finishSubscription();
    await new Promise(resolve => setTimeout(resolve, 0));
    bridge.destroy();
    return { error, disposed, snapshots };
  });
  expect(result).toEqual({ error: 'subscription failed', disposed: 1, snapshots: 0 });
});

test('destroy during initial snapshot rejects readiness without late layout work', async ({ page }) => {
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await page.waitForFunction(() => window.__RADIO_UI_READY__ === true);
  const result = await page.evaluate(async () => {
    window.radioPreview.ui.destroy(); window.radioPreview.bridge.destroy();
    const { createRadioUI } = await import('/radio.js');
    const { createPreviewBridge } = await import('/preview-bridge.js');
    const bridge = createPreviewBridge(), snapshot = await bridge.getSnapshot();
    let finishSnapshot, snapshotRequested;
    const requested = new Promise(resolve => { snapshotRequested = resolve; });
    bridge.getSnapshot = () => { snapshotRequested(); return new Promise(resolve => { finishSnapshot = resolve; }); };
    const ui = createRadioUI(document.querySelector('#radio'), bridge);
    const outcome = ui.ready.then(() => 'unexpected success', error => error.message);
    await requested;
    ui.destroy(); finishSnapshot(snapshot);
    const error = await outcome;
    bridge.destroy();
    return { error, layouts: bridge.layouts.length };
  });
  expect(result).toEqual({ error: 'Radio UI was destroyed during startup', layouts: 0 });
  expect(errors).toEqual([]);
});

test('spectrum uses public lifecycle for veto, disable and destroyed mutations', async ({ page }) => {
  await page.goto('/');
  await page.waitForFunction(() => window.__RADIO_UI_READY__ === true);
  const result = await page.evaluate(async () => {
    const { SpectrumAnalyzer } = await import('/extensions/SpectrumAnalyzer.js');
    const { LifecycleError } = await import('/vendor/taet-ui.js');
    const spectrum = new SpectrumAnalyzer(document.body, { name: 'publicLifecycleSpectrum' });
    const veto = spectrum.events.on('beforeHide', event => { event.cancel = true; });
    spectrum.hide(); const vetoed = spectrum.visible;
    veto.dispose(); spectrum.hide(); const hidden = !spectrum.visible;
    spectrum.show(); spectrum.disable(); const disabled = spectrum.disabled;
    spectrum.enable(); const enabled = !spectrum.disabled;
    spectrum.disposables.add(() => spectrum.destroy()); // reentrant teardown stays idempotent
    spectrum.destroy(); spectrum.destroy();
    const mutations = { setLevels: () => spectrum.setLevels([0, 0, 0, 0, 0]), clear: () => spectrum.clear(), setBandCount: () => spectrum.setBandCount(5), setRowCount: () => spectrum.setRowCount(13), setMode: () => spectrum.setMode('line'), setPaused: () => spectrum.setPaused(true), setColors: () => spectrum.setColors({}) };
    const rejected = Object.entries(mutations).every(([operation, mutate]) => {
      try { mutate(); return false; } catch (error) { return error instanceof LifecycleError && error.controlId === spectrum.id && error.operation === operation; }
    });
    return { vetoed, hidden, disabled, enabled, rejected, destroyed: spectrum.destroyed };
  });
  expect(result).toEqual({ vetoed: true, hidden: true, disabled: true, enabled: true, rejected: true, destroyed: true });
});
