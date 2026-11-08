import { test, expect } from '@playwright/test';
const control = (page, name) => page.locator(`.player-strip [data-control="${name}"]`);
const state = page => page.evaluate(() => window.radioPreview.ui.getState());
test.beforeEach(async ({ page }) => { await page.goto('/'); await page.waitForFunction(() => window.__RADIO_UI_READY__ === true); });

test('toolbar SVG identity, contrast, alignment and unchanged hit targets', async ({ page }) => {
  const names = { collapse: 'collapse', play: 'play', mute: 'speaker', settings: 'settings', equalizer: 'music', close: 'close' };
  const metrics = await page.evaluate(async expected => {
    const { icons } = await import('/vendor/taet-ui.js');
    return Object.entries(expected).map(([name, icon]) => {
      const button = document.querySelector(`[data-control="${name}"]`), svg = button.querySelector('svg'), shape = svg.firstElementChild;
      const b = button.getBoundingClientRect(), s = svg.getBoundingClientRect(), definition = icons.resolve(`radio-${icon}`);
      return { name, hit: { width: b.width, height: b.height }, size: { width: s.width, height: s.height }, centerX: s.x + s.width / 2 - b.x - b.width / 2, centerY: s.y + s.height / 2 - b.y - b.height / 2,
        matchesDefinition: svg.getAttribute('viewBox') === definition.viewBox && shape.tagName === definition.nodes[0][0] && Object.entries(definition.nodes[0][1]).every(([key, value]) => shape.getAttribute(key) === String(value)),
        color: getComputedStyle(button).color, iconColor: getComputedStyle(svg).color, background: getComputedStyle(button).backgroundColor, title: button.title, label: button.getAttribute('aria-label'), hiddenFromAT: svg.getAttribute('aria-hidden') };
    });
  }, names);
  for (const m of metrics) {
    expect(m.matchesDefinition, `${m.name} icon mapping`).toBe(true);
    expect(m.hit).toEqual({ width: m.name === 'collapse' ? 18 : 26, height: 26 });
    expect(m.size).toEqual({ width: m.name === 'collapse' ? 13 : 18, height: m.name === 'collapse' ? 16 : 18 });
    expect(Math.abs(m.centerX)).toBeLessThanOrEqual(.75); expect(Math.abs(m.centerY)).toBeLessThanOrEqual(.75);
    expect(m.iconColor).toBe(m.color); expect(m.color).toBe('rgb(226, 230, 234)');
    expect(m.background).toBe('rgb(0, 0, 0)'); expect(m.title).toBe(''); expect(m.label).toBeTruthy(); expect(m.hiddenFromAT).toBe('true');
  }
  expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({ width: 725, height: 36 });
  const mute = control(page, 'mute'); await mute.click();
  await expect(mute).toHaveAttribute('aria-label', 'Unmute'); await expect(mute).not.toHaveAttribute('title');
  expect(await mute.locator('svg').getAttribute('viewBox')).toBe('4 2 16 20');
  await control(page, 'play').click(); await expect(control(page, 'play').locator('svg rect')).toHaveCount(1);
  await control(page, 'collapse').click();
  expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({ width: 102, height: 36 });
  await expect(page.locator('.player-strip > :visible')).toHaveCount(3);
});

test('panel pressed state tracks visibility through keyboard, switch, close and compact', async ({ page }) => {
  const settings = control(page, 'settings'), eq = control(page, 'equalizer'), mute = control(page, 'mute');
  const check = async (s, e) => {
    await expect(settings).toHaveAttribute('aria-pressed', String(s)); await expect(settings).toHaveAttribute('aria-expanded', String(s));
    await expect(eq).toHaveAttribute('aria-pressed', String(e)); await expect(eq).toHaveAttribute('aria-expanded', String(e));
    if (s) await expect(page.locator('.settings-panel')).toBeVisible(); else await expect(page.locator('.settings-panel')).toBeHidden();
    if (e) await expect(page.locator('.eq-panel')).toBeVisible(); else await expect(page.locator('.eq-panel')).toBeHidden();
  };
  await check(false, false); const normal = await settings.evaluate(e => getComputedStyle(e).backgroundColor);
  await settings.click(); await check(true, false); await settings.click(); await check(false, false);
  await page.mouse.move(850, 500); expect(await settings.evaluate(e => getComputedStyle(e).backgroundColor)).toBe(normal);
  await settings.focus(); await page.keyboard.press('Enter'); await check(true, false);
  await expect(settings).toHaveCSS('outline-width', '2px');
  await page.keyboard.press('Space'); await check(false, false); await page.keyboard.press('Enter'); await check(true, false);
  await eq.click(); await check(false, true);
  const deleteButton = page.getByRole('button', { name: 'Delete', exact: true }); await expect(deleteButton).toBeDisabled();
  const disabled = await deleteButton.evaluate(e => getComputedStyle(e).backgroundColor); await deleteButton.hover();
  expect(await deleteButton.evaluate(e => getComputedStyle(e).backgroundColor)).toBe(disabled);
  await mute.click(); await expect(mute).toHaveAttribute('aria-pressed', 'true'); await check(false, true);
  expect(await mute.evaluate(e => getComputedStyle(e).backgroundColor)).not.toBe(await eq.evaluate(e => getComputedStyle(e).backgroundColor));
  await page.evaluate(() => window.radioPreview.ui.showPanel(null)); await check(false, false);
  await settings.click(); const editor = page.locator('[data-station-slot="5"]');
  await editor.fill('https://synthetic.invalid/draft?quote=%22&value=%D0%96'); const draft = await editor.inputValue();
  const generation = (await state(page)).snapshot.generation;
  await control(page, 'collapse').click(); await check(false, false);
  await control(page, 'collapse').press('Enter'); await check(true, false); await expect(editor).toHaveValue(draft);
  await eq.click(); await control(page, 'collapse').click(); await check(false, false);
  await control(page, 'collapse').press('Space'); await check(false, true);
  expect((await state(page)).snapshot.generation).toBe(generation);
  const actions = await page.evaluate(() => window.radioPreview.bridge.actions);
  expect(actions.filter(a => ['setPlaying', 'selectStation'].includes(a.type))).toEqual([]);
});


test('hover hints stay absent across panels, playback labels, metadata and errors', async ({ page }) => {
  const noHints = async () => {
    await expect(page.locator('.radio-owner [title]:not([title=""])')).toHaveCount(0);
    await expect(page.locator('.radio-owner .dui-trackbar--show-value')).toHaveCount(0);
  };
  await noHints();
  await control(page, 'play').click();
  await expect(control(page, 'play')).toHaveAccessibleName('Stop');
  await control(page, 'mute').click();
  await expect(control(page, 'mute')).toHaveAccessibleName('Unmute');
  await noHints();
  await page.evaluate(() => window.radioPreview.bridge.update({ status: 'playing', requestedPlaying: true, displayTitle: 'Artist - Title', fullTitle: 'Artist - Full title' }));
  await expect(page.locator('.radio-ticker')).toHaveAttribute('aria-label', /Artist - Title/);
  await noHints();
  await control(page, 'settings').click();
  const url = page.locator('[data-station-slot="0"]');
  await url.focus(); await expect(url).toHaveAccessibleName('Station 1 URL');
  await url.press('Tab');
  await page.getByRole('button', { name: 'Browse bank Z', exact: true }).click();
  await noHints();
  await control(page, 'equalizer').click();
  const gain = page.getByRole('slider', { name: 'Preamp gain', exact: true });
  await gain.hover(); await gain.focus(); await gain.press('ArrowUp');
  await expect(gain).toHaveValue('1');
  await expect(page.locator('.eq-sliders .dui-trackbar__value').first()).toBeHidden();
  await noHints();
  await page.evaluate(() => window.radioPreview.bridge.update({ error: 'Synthetic certificate failure' }));
  await expect(page.locator('.radio-ticker')).toHaveAttribute('aria-label', /Error: .*Synthetic certificate failure/);
  await expect(control(page, 'play')).toHaveAttribute('aria-description', 'Synthetic certificate failure');
  await control(page, 'collapse').click();
  await expect(control(page, 'collapse')).toHaveAccessibleName('Expand player');
  await noHints();
  await control(page, 'collapse').click(); await noHints();
});
