import { Button, IconButton, CommandRegistry, Form, Panel, ReadonlyText, ScrollingLabel, SelectInput, TextInput, TrackBar } from './vendor/taet-ui.js';
import { ChannelIndicator } from './channel-indicator.js';
import { registerRadioIcons } from './icons.js';
import { SpectrumAnalyzer } from './extensions/SpectrumAnalyzer.js';
const letter = bank => String.fromCharCode(65 + bank);
const digit = slot => String((slot + 1) % 10);
const silent = { silent: true, origin: 'import' };
const isEditor = target => target instanceof Element && !!target.closest('input,textarea,select,[contenteditable="true"],[role="combobox"],[role="slider"],[role="listbox"]');

/** One owner, one bridge, persistent live controls. No audio or persistence in JS. */
export function createRadioUI(host, bridge) {
  registerRadioIcons();
  let snapshot = null, browsedBank = 0, bankLoaded = -1, compact = false, panel = null, lastPanel = null, destroyed = false;
  let eqLoading = false, presetSignature = '', errorText = '', lastLayout = '';
  let layoutPromise = Promise.resolve();
  const drafts = new Map(), cleanup = [], bankButtons = [], rows = [], bandControls = [];
  const owner = new Panel(host, { name: 'radioOwner', css: 'radio-owner', padding: 0 });
  const strip = new Panel({ name: 'strip', css: 'player-strip', padding: 0 }); owner.add(strip);
  const commands = new CommandRegistry();
  const setLabel = (control, name) => { if (control instanceof IconButton) control.tooltip = name; control.element?.setAttribute('aria-label', name); control.element?.setAttribute('title', name); const focus = control.getFocusElement(); focus?.setAttribute('aria-label', name); };
  const addButton = (parent, name, text, label, click) => {
    const button = new Button({ name, text, onClick: click }); parent.add(button); setLabel(button, label); button.element.dataset.control = name; return button;
  };
  const addIconButton = (name, icon, label, click, pressed = null) => {
    const button = new IconButton({ name, icon: `radio-${icon}`, tooltip: label, pressed, onClick: click });
    strip.add(button); setLabel(button, label); button.element.dataset.control = name; return button;
  };
  const report = error => { if (!destroyed) { errorText = String(error?.message || error); renderStatus(); } };
  const run = action => Promise.resolve(bridge.dispatch(action)).then(next => { errorText = ''; return applySnapshot(next); }).catch(error => { report(error); return null; });
  const command = (id, fn) => { commands.register(id, { run: fn }); return () => commands.execute(id); };
  const chevron = addIconButton('collapse', 'collapse', 'Collapse player', () => setCompact(!compact));
  const play = addIconButton('play', 'play', 'Play', command('play', () => snapshot && run({ type: 'setPlaying', playing: !snapshot.requestedPlaying })), false);
  const display = new Panel({ name: 'display', css: 'radio-display', padding: 0 }); strip.add(display);
  const spectrum = new SpectrumAnalyzer({ name: 'spectrum', bands: 5, width: 42, height: 24, background: '#003D66', idleColor: '#323940', baselineColor: '#61a77e', activeColors: ['#74b88b', '#b9c762', '#da8664'], peakColor: '#d7e3b1' }); display.add(spectrum);
  spectrum.element.classList.add('radio-spectrum'); spectrum.element.dataset.dragSurface = 'true';
  const ticker = new ScrollingLabel({ name: 'ticker', text: 'Æ Strøm', speed: 28, startDelay: 1200, className: 'radio-ticker' }); display.add(ticker);
  const indicator = new ChannelIndicator({ name: 'channel' }); display.add(indicator);
  const mute = addIconButton('mute', 'speaker', 'Mute', command('mute', () => snapshot && run({ type: 'setMuted', muted: !snapshot.muted })), false);
  const volume = new TrackBar({ name: 'volume', thumbShape: 'grip', min: 0, max: 100, step: 1, smallChange: 5, largeChange: 10, value: 80, showValue: false, onChange: e => run({ type: 'setVolume', volume: e.newValue }) }); strip.add(volume); setLabel(volume, 'Volume'); volume.element.classList.add('radio-volume');
  const settingsButton = addIconButton('settings', 'settings', 'Station settings', () => showPanel('settings'), false);
  const eqButton = addIconButton('equalizer', 'music', 'Equalizer', () => showPanel('eq'), false);
  addIconButton('close', 'close', 'Close player', () => Promise.resolve(bridge.shutdown()).catch(report));
  const settings = new Panel({ name: 'settingsPanel', css: 'settings-panel', padding: 8 }); owner.add(settings);
  const columns = new Panel({ name: 'stationColumns', css: 'station-columns', padding: 0 }); settings.add(columns);
  for (let col = 0; col < 2; col++) {
    const column = new Panel({ name: `stationColumn${col}`, css: 'station-column', padding: 0 }); columns.add(column);
    const header = new Form({ name: `stationHeader${col}`, css: 'station-header', labelWidth: 0 }).row(
      new ReadonlyText({ name: `keyLabel${col}`, text: 'Key', className: 'column-label column-key' }),
      new ReadonlyText({ name: `urlLabel${col}`, text: 'Stream URL', className: 'column-label column-url' }),
      new ReadonlyText({ name: `eqLabel${col}`, text: 'EQ Preset', className: 'column-label column-eq' })
    ); column.add(header);
    for (let row = 0; row < 5; row++) {
      const slot = col * 5 + row;
      const marker = new Button({ name: `stationMarker${slot}`, text: digit(slot), onClick: () => run({ type: 'selectStation', bank: browsedBank, slot }) });
      const url = new TextInput({ name: `stationUrl${slot}`, label: '', value: '', inputType: 'text', spellcheck: false, autocomplete: 'off', onChange: e => drafts.set(`${browsedBank}:${slot}`, e.newValue), onBlur: () => commitUrl(slot), onKeyDown: e => { if (e.key === 'Enter' && !e.nativeEvent.isComposing) commitUrl(slot); } });
      const eq = new SelectInput({ name: `stationEq${slot}`, valueField: 'name', textField: 'name', value: '', searchable: false, popupWidth: 180, onChange: e => run({ type: 'assignStationEq', bank: browsedBank, slot, name: String(e.newValue ?? '') }) });
      const form = new Form({ name: `stationRow${slot}`, css: 'station-row', labelWidth: 0 }).row(marker, url, eq); column.add(form);
      setLabel(marker, `Play station ${digit(slot)}`); marker.element.setAttribute('role', 'radio');
      setLabel(url, `Station ${digit(slot)} URL`); url.getFocusElement().dataset.stationSlot = String(slot);
      setLabel(eq, `Station ${digit(slot)} equalizer`);
      rows.push({ slot, marker, url, eq });
    }
  }
  const banks = new Panel({ name: 'banks', css: 'bank-strip', padding: 0 }); settings.add(banks);
  for (let bank = 0; bank < 26; bank++) bankButtons.push(addButton(banks, `bank${bank}`, letter(bank), `Browse bank ${letter(bank)}`, () => browseBank(bank)));
  const eqPanel = new Panel({ name: 'eqPanel', css: 'eq-panel', padding: 8 }); owner.add(eqPanel);
  const preset = new SelectInput({ name: 'preset', valueField: 'name', textField: 'name', value: 'Flat', onChange: e => run({ type: 'selectEqPreset', name: String(e.newValue) }) });
  const presetName = new TextInput({ name: 'presetName', value: '', placeholder: 'Custom preset name', spellcheck: false, autocomplete: 'off' });
  const savePreset = new Button({ name: 'savePreset', text: 'Save', onClick: () => run({ type: 'saveEqPreset', name: presetName.value.get().trim() }) });
  const deletePreset = new Button({ name: 'deletePreset', text: 'Delete', onClick: () => snapshot && run({ type: 'deleteEqPreset', name: snapshot.eq.name }) });
  const eqTools = new Form({ name: 'eqTools', css: 'eq-tools' }).row(preset, presetName, savePreset, deletePreset); eqPanel.add(eqTools);
  setLabel(preset, 'Equalizer preset'); setLabel(presetName, 'Custom preset name');
  const eqSliders = new Panel({ name: 'eqSliders', css: 'eq-sliders', padding: 0 }); eqPanel.add(eqSliders);
  const eqGrid = new Panel({ name: 'eqGrid', css: 'eq-grid', padding: 0 }); eqSliders.add(eqGrid);
  eqGrid.element.setAttribute('aria-hidden', 'true');
  const eqGridLines = [15, 10, 5, 0, -5, -10, -15].map(db => {
    const major = db === 15 || db === 0 || db === -15;
    const line = new Panel({ name: `eqGridLine${db}`, css: `eq-grid-line${major ? ' eq-grid-line--major' : ''}`, padding: 0 });
    eqGrid.add(line); line.element.dataset.eqDb = String(db);
    line.add(new ReadonlyText({ name: `eqScaleLabel${db}`, text: `${db > 0 ? '+' : ''}${db}${major ? ' dB' : ''}`, className: 'eq-grid-label' }));
    return { db, line };
  });
  const frequencies = ['Preamp', '32', '64', '125', '250', '500', '1K', '2K', '4K', '8K', '16KHz'];
  for (let i = 0; i < 11; i++) {
    const slider = new TrackBar({ name: `eqBand${i}`, label: frequencies[i], labelPosition: 'top', min: -15, max: 15, step: 1, value: 0, orientation: 'vertical', invert: false, scale: 'none', showTickLabels: false, thumbShape: 'grip', showValue: true, valuePosition: 'top', valueFormatter: value => `${value > 0 ? '+' : ''}${value} dB`, onChange: () => { if (!eqLoading) run({ type: 'setEq', preamp: bandControls[0].value.get(), bands: bandControls.slice(1).map(c => c.value.get()) }); } });
    eqSliders.add(slider); setLabel(slider, i ? `${frequencies[i].replace('Hz', '')} Hz gain` : 'Preamp gain'); slider.element.removeAttribute('title'); bandControls.push(slider);
    const input = slider.getFocusElement();
    const beginDrag = () => { slider.element.dataset.dragging = 'true'; };
    const endDrag = () => { delete slider.element.dataset.dragging; };
    input.addEventListener('pointerdown', beginDrag);
    document.addEventListener('pointerup', endDrag); document.addEventListener('pointercancel', endDrag); window.addEventListener('blur', endDrag);
    cleanup.push(() => input.removeEventListener('pointerdown', beginDrag), () => document.removeEventListener('pointerup', endDrag), () => document.removeEventListener('pointercancel', endDrag), () => window.removeEventListener('blur', endDrag));
  }
  // The native range thumb travels inside the input by half its actual CSS
  // grip height and the explicit track border at either end.
  function updateEqGrid() {
    if (destroyed || eqPanel.element.hidden || compact) return;
    const inputs = bandControls.map(control => control.getFocusElement());
    const rects = inputs.map(input => input.getBoundingClientRect()), bounds = eqSliders.element.getBoundingClientRect();
    if (!rects[0].height) return;
    const centerX = rect => rect.x + rect.width / 2;
    const startX = centerX(rects[0]), endX = centerX(rects[10]);
    const halfThumb = parseFloat(getComputedStyle(bandControls[0].element).getPropertyValue('--dui-thumb-height')) / 2;
    const inset = halfThumb + parseFloat(getComputedStyle(eqSliders.element).getPropertyValue('--radio-eq-track-border'));
    for (const { db, line } of eqGridLines) {
      const y = rects[0].top - bounds.top + inset + (15 - db) / 30 * (rects[0].height - 2 * inset);
      line.element.style.left = `${startX - bounds.left}px`; line.element.style.width = `${endX - startX}px`; line.element.style.top = `${y}px`;
      line.items[0].element.style.left = `${(centerX(rects[1]) - startX) / 2}px`;
    }
  }
  const eqResize = new ResizeObserver(updateEqGrid); eqResize.observe(eqSliders.element);
  window.addEventListener('resize', updateEqGrid);
  cleanup.push(() => eqResize.disconnect(), () => window.removeEventListener('resize', updateEqGrid));
  const status = new ReadonlyText({ name: 'status', text: '', className: 'radio-status', role: 'status' }); owner.add(status);
  settings.hide(); eqPanel.hide();
  status.element.setAttribute('aria-live', 'polite');
  status.element.setAttribute('aria-atomic', 'true');
  function renderStatus() {
    const message = errorText || snapshot?.error || '';
    status.setText(message);
    const fullTitle = snapshot?.fullTitle || 'Æ Strøm';
    const title = String(message || snapshot?.displayTitle || fullTitle).slice(0, 240);
    ticker.setText(title);
    ticker.element.title = `${snapshot?.mode === 'fake' ? 'Synthetic preview - no audio | ' : ''}${message || fullTitle} | ${snapshot?.status || 'stopped'}`;
    ticker.element.setAttribute('aria-label', `${message ? 'Error: ' : ''}${snapshot?.mode === 'fake' ? 'Synthetic preview. No audio. ' : ''}${title}`);
    for (const control of [chevron, play]) {
      control.element.setAttribute('aria-description', message);
      control.element.title = `${control.element.getAttribute('aria-label')}${message ? ': ' + message : ''}`;
    }
    requestWindowLayout();
  }
  function requestWindowLayout() {
    const width = compact ? Math.ceil(strip.element.scrollWidth) : 725;
    const height = Math.ceil(owner.element.getBoundingClientRect().height);
    const key = `${compact}:${width}:${height}`;
    if (key === lastLayout) return layoutPromise;
    lastLayout = key;
    layoutPromise = Promise.resolve(bridge.setWindowLayout({ compact, width, height })).catch(report);
    return layoutPromise;
  }
  function renderLayout() {
    owner.element.classList.toggle('compact', compact);
    owner.element.dataset.panel = panel || '';
    for (const child of strip.items) { if ([chevron, play, display].includes(child)) continue; compact ? child.hide() : child.show(); }
    for (const child of [ticker, indicator]) compact ? child.hide() : child.show();
    !compact && panel === 'settings' ? settings.show() : settings.hide();
    !compact && panel === 'eq' ? eqPanel.show() : eqPanel.hide();
    chevron.setIcon(compact ? 'radio-expand' : 'radio-collapse'); setLabel(chevron, compact ? 'Expand player' : 'Collapse player'); chevron.element.setAttribute('aria-expanded', String(!compact));
    const settingsVisible = !compact && panel === 'settings', eqVisible = !compact && panel === 'eq';
    settingsButton.setPressed(settingsVisible); eqButton.setPressed(eqVisible);
    settingsButton.element.setAttribute('aria-expanded', String(settingsVisible)); eqButton.element.setAttribute('aria-expanded', String(eqVisible));
    updateEqGrid(); renderStatus();
  }
  function setCompact(value) {
    compact = value;
    if (compact && host.contains(document.activeElement) && ![chevron.getFocusElement(), play.getFocusElement()].includes(document.activeElement)) chevron.focus();
    renderLayout();
  }
  function showPanel(next) { compact = false; panel = panel === next ? null : next; if (panel) lastPanel = panel; renderLayout(); }
  function browseBank(bank) { if (!snapshot) return; browsedBank = bank; loadBank(); }
  function loadBank() {
    bankLoaded = browsedBank;
    bankButtons.forEach((button, bank) => button.element.setAttribute('aria-pressed', String(bank === browsedBank)));
    for (const row of rows) { const entry = snapshot.stations[browsedBank * 10 + row.slot]; row.url.value.set(drafts.get(`${browsedBank}:${row.slot}`) ?? entry.url, silent); row.eq.value.set(entry.eq || '', silent); }
    renderMarkers();
  }
  function renderMarkers() {
    for (const row of rows) row.marker.element.setAttribute('aria-checked', String(snapshot.activeBank === browsedBank && snapshot.activeSlot === row.slot));
  }
  function commitUrl(slot) {
    if (!snapshot || destroyed) return;
    const bank = browsedBank, key = `${bank}:${slot}`, row = rows.find(r => r.slot === slot), url = row.url.value.get();
    if (snapshot.stations[bank * 10 + slot].url === url) { drafts.delete(key); return; }
    run({ type: 'setStationUrl', bank, slot, url }).then(saved => { if (saved && drafts.get(key) === url) drafts.delete(key); });
  }
  function applySnapshot(next) {
    if (destroyed || !next || snapshot && next.revision < snapshot.revision) return;
    const previous = snapshot; snapshot = next;
    play.setIcon(next.requestedPlaying ? 'radio-stop' : 'radio-play'); setLabel(play, next.requestedPlaying ? 'Stop' : 'Play');
    play.setPressed(next.requestedPlaying); mute.setPressed(next.muted); mute.setIcon(next.muted ? 'radio-muted' : 'radio-speaker'); setLabel(mute, next.muted ? 'Unmute' : 'Mute');
    volume.value.set(next.volume, silent);
    owner.element.dataset.mode = next.mode;
    indicator.setChannel(next.activeBank, next.activeSlot, Boolean(previous && (previous.activeBank !== next.activeBank || previous.activeSlot !== next.activeSlot)));
    if (!next.requestedPlaying || next.status !== 'playing') spectrum.clear();
    const signature = JSON.stringify(next.eqPresets.map(p => p.name));
    if (signature !== presetSignature) { presetSignature = signature; preset.data.parse(next.eqPresets); for (const row of rows) row.eq.data.parse([{ name: '' }, ...next.eqPresets]); }
    preset.value.set(next.eq.name, silent);
    eqLoading = true; bandControls.forEach((control, i) => control.value.set(i ? next.eq.bands[i - 1] : next.eq.preamp, silent)); eqLoading = false;
    next.eq.canDelete ? deletePreset.enable() : deletePreset.disable();
    if (bankLoaded < 0) { browsedBank = next.activeBank; loadBank(); } else renderMarkers();
    for (const row of rows) {
      const entry = next.stations[browsedBank * 10 + row.slot], key = `${browsedBank}:${row.slot}`;
      if (!drafts.has(key) && document.activeElement !== row.url.getFocusElement() && row.url.value.get() !== entry.url) row.url.value.set(entry.url, silent);
      if (document.activeElement !== row.eq.getFocusElement()) row.eq.value.set(entry.eq || '', silent);
    }
    renderStatus();
    return next;
  }
  function keyboard(event) {
    if (destroyed || event.defaultPrevented || event.isComposing || event.repeat || isEditor(event.target)) return;
    if (event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey && event.code === 'KeyC') { event.preventDefault(); Promise.resolve(bridge.copyTitle()).catch(report); return; }
    if (!snapshot || event.ctrlKey || event.altKey || event.metaKey) return;
    let action;
    if (/^Key[A-Z]$/.test(event.code) && !event.shiftKey) action = { type: 'selectStation', bank: event.code.charCodeAt(3) - 65, slot: snapshot.activeSlot };
    else if (/^(Digit|Numpad)[0-9]$/.test(event.code) && !event.shiftKey) action = { type: 'selectStation', bank: snapshot.activeBank, slot: (Number(event.code.slice(-1)) + 9) % 10 };
    else if (event.code === 'NumpadAdd' || event.key === '+') action = { type: 'setVolume', volume: Math.min(100, snapshot.volume + 5) };
    else if (event.code === 'NumpadSubtract' || event.key === '-') action = { type: 'setVolume', volume: Math.max(0, snapshot.volume - 5) };
    else if (event.code === 'NumpadMultiply' || event.key === '*') action = { type: 'setMuted', muted: !snapshot.muted };
    if (action) { event.preventDefault(); run(action); }
  }
  let dragStart = null;
  const interactiveDragTarget = 'button,input,textarea,select,a,[contenteditable="true"],[role="button"],[role="slider"],[role="combobox"],[role="listbox"],[role="menu"],[role="menuitem"],.radio-volume,[data-no-window-drag]';
  function pointerDown(event) {
    dragStart = null;
    const target = event.target instanceof Element ? event.target : null;
    if (event.button !== 0 || event.defaultPrevented || !target || target.closest(interactiveDragTarget)) return;
    // One listener on the strip: nested ticker/channel content routes once.
    dragStart = { x: event.clientX, y: event.clientY, pointerId: event.pointerId };
  }
  function pointerMove(event) {
    if (!dragStart || event.pointerId !== dragStart.pointerId) return;
    if (event.buttons !== 1) { dragStart = null; return; }
    if (Math.hypot(event.clientX - dragStart.x, event.clientY - dragStart.y) < 4) return;
    dragStart = null;
    event.preventDefault();
    Promise.resolve(bridge.startDrag()).catch(report);
  }
  function pointerUp() { dragStart = null; }
  document.addEventListener('keydown', keyboard); strip.element.addEventListener('pointerdown', pointerDown); document.addEventListener('pointermove', pointerMove); document.addEventListener('pointerup', pointerUp); document.addEventListener('pointercancel', pointerUp); window.addEventListener('blur', pointerUp);
  cleanup.push(() => document.removeEventListener('keydown', keyboard), () => strip.element.removeEventListener('pointerdown', pointerDown), () => document.removeEventListener('pointermove', pointerMove), () => document.removeEventListener('pointerup', pointerUp), () => document.removeEventListener('pointercancel', pointerUp), () => window.removeEventListener('blur', pointerUp));
  // Subscribe before requesting the snapshot; revision/generation reject stale delivery.
  const listen = (method, fn) => Promise.resolve(method.call(bridge, fn)).then(off => { if (destroyed) off?.(); else cleanup.push(off); }).catch(report);
  const stateSubscription = listen(bridge.subscribeState, applySnapshot);
  const spectrumSubscription = listen(bridge.subscribeSpectrum, event => { if (destroyed || !snapshot || event.generation !== snapshot.generation) return; if (!event.powered || !snapshot.requestedPlaying || snapshot.status !== 'playing') spectrum.clear(); else spectrum.setLevels(event.levels); });
  const ready = Promise.all([stateSubscription, spectrumSubscription]).then(() => bridge.getSnapshot()).then(applySnapshot).then(async () => {
    await document.fonts.load('12px "Noto Sans"', 'Æ Strøm Жж');
    await document.fonts.ready;
    renderLayout();
    await requestWindowLayout();
  }).catch(report);
  return { ready, getState: () => ({ snapshot, compact, panel, lastPanel, browsedBank, drafts: Object.fromEntries(drafts) }), setCompact, showPanel, destroy() { if (destroyed) return; destroyed = true; cleanup.splice(0).forEach(fn => fn?.()); commands.destroy(); owner.destroy(); } };
}

