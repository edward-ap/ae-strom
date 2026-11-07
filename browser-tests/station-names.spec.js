import { test, expect } from '@playwright/test';
const field = (page, slot = 0) => page.locator(`[data-station-slot="${slot}"]`);
const saves = page => page.evaluate(() => window.radioPreview.bridge.actions.filter(a => a.type === 'setStationUrl'));
const station = (page, patch, index = 0) => page.evaluate(({ patch, index }) => {
  const { ui, bridge } = window.radioPreview, stations = structuredClone(ui.getState().snapshot.stations);
  Object.assign(stations[index], patch); bridge.update({ stations });
}, { patch, index });
test.beforeEach(async ({ page }) => {
  await page.goto('/'); await page.waitForFunction(() => window.__RADIO_UI_READY__ === true);
  await page.getByRole('button', { name: 'Station settings', exact: true }).click();
});

test('station name switches to URL for mouse and keyboard focus without saving', async ({ page }, testInfo) => {
  const input = field(page);
  await expect(input).toHaveValue('Synthetic station 1');
  await input.click(); await expect(input).toHaveValue('https://synthetic.invalid/station/1');
  await input.press('Enter'); await input.press('Tab'); await expect(input).toHaveValue('Synthetic station 1');
  await page.keyboard.press('Shift+Tab'); await expect(input).toBeFocused();
  await expect(input).toHaveValue('https://synthetic.invalid/station/1');
  await input.press('Tab');
  expect(await saves(page)).toEqual([]);
  expect(await page.evaluate(() => window.radioPreview.ui.getState().drafts)).toEqual({});
  await page.screenshot({ path: testInfo.outputPath('settings-station-names.png') });
});

test('metadata updates inactive labels while focused URL selection remains intact', async ({ page }) => {
  const input = field(page);
  await input.focus(); await input.evaluate(e => e.setSelectionRange(8, 19, 'backward'));
  await station(page, { name: 'Discovered Radio' });
  await expect(input).toHaveValue('https://synthetic.invalid/station/1');
  expect(await input.evaluate(e => [e.selectionStart, e.selectionEnd, e.selectionDirection])).toEqual([8, 19, 'backward']);
  await input.press('Tab'); await expect(input).toHaveValue('Discovered Radio');
  await station(page, { name: 'Updated Radio' }); await expect(input).toHaveValue('Updated Radio');
  await station(page, { name: '' }); await expect(input).toHaveValue('https://synthetic.invalid/station/1');
  expect(await saves(page)).toEqual([]);
});

test('editing saves only the URL and discovery restores the name view', async ({ page }) => {
  const input = field(page), url = 'https://synthetic.invalid/new-stream?q=%D0%96';
  await input.fill(url); await input.press('Tab');
  await expect.poll(() => saves(page)).toEqual([{ type: 'setStationUrl', bank: 0, slot: 0, url }]);
  await expect(input).toHaveValue(url);
  await station(page, { name: 'New Radio' }); await expect(input).toHaveValue('New Radio');
  await input.focus(); await expect(input).toHaveValue(url);
  await input.press('Tab'); expect((await saves(page)).length).toBe(1);
});

test('pending Enter and blur save once and a stale reply does not retain a fake draft', async ({ page }) => {
  await page.evaluate(() => {
    const bridge = window.radioPreview.bridge, dispatch = bridge.dispatch.bind(bridge);
    window.urlRequests = [];
    bridge.dispatch = action => action.type !== 'setStationUrl' ? dispatch(action) : new Promise(resolve => {
      window.urlRequests.push(action);
      window.finishUrl = async () => {
        const saved = await dispatch(action);
        const stations = structuredClone(saved.stations); stations[0].name = 'Discovered During Save';
        bridge.update({ stations }); // newer state arrives before the older invoke reply
        resolve(saved);
      };
    });
  });
  const input = field(page), url = 'https://synthetic.invalid/pending';
  await input.fill(url); await input.press('Enter'); await input.press('Tab');
  expect(await page.evaluate(() => window.urlRequests.length)).toBe(1);
  await page.getByRole('button', { name: 'Browse bank Z', exact: true }).click();
  await expect(input).toHaveValue('Synthetic station 251');
  await page.evaluate(() => window.finishUrl());
  await expect(input).toHaveValue('Synthetic station 251');
  await page.getByRole('button', { name: 'Browse bank A', exact: true }).click();
  await expect(input).toHaveValue('Discovered During Save');
  await input.focus(); await expect(input).toHaveValue(url);
  expect(await page.evaluate(() => window.radioPreview.ui.getState().drafts)).toEqual({});
});

test('a rejected URL remains available for correction and cannot become a station-name save', async ({ page }) => {
  await page.evaluate(() => {
    const bridge = window.radioPreview.bridge, dispatch = bridge.dispatch.bind(bridge);
    bridge.dispatch = async action => {
      if (action.type === 'setStationUrl') throw new Error('Invalid stream URL');
      return dispatch(action);
    };
  });
  const input = field(page); await input.fill('invalid URL'); await input.press('Tab');
  await expect(page.locator('.radio-status')).toHaveText('Invalid stream URL');
  await expect(input).toHaveValue('invalid URL');
  await input.focus(); await expect(input).toHaveValue('invalid URL');
  expect(await page.evaluate(() => window.radioPreview.ui.getState().snapshot.stations[0].url)).toBe('https://synthetic.invalid/station/1');
  expect(await page.evaluate(() => window.radioPreview.ui.getState().snapshot.stations[0].name)).toBe('Synthetic station 1');
});
