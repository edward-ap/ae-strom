import { createRadioUI } from './radio.js';
import { createTauriBridge } from './bridge.js';
import { createPreviewBridge } from './preview-bridge.js';
const bridge = window.__TAURI__ ? createTauriBridge() : createPreviewBridge();
const ui = createRadioUI(document.querySelector('#radio'), bridge);
window.addEventListener('pagehide', () => { ui.destroy(); bridge.destroy?.(); }, { once: true });
// Deliberate inspection hook for the isolated synthetic acceptance harness.
if (!window.__TAURI__) window.radioPreview = { ui, bridge };

const ready = ui.ready.then(async () => {
  const bounds = document.querySelector('.radio-owner').getBoundingClientRect();
  const fontLoaded = document.fonts.check('12px "Noto Sans"', 'Æ Strøm Жж') && [...document.fonts].some(face => face.family.includes('Noto Sans') && face.status === 'loaded');
  await bridge.uiReady({ width: bounds.width, height: bounds.height, fontLoaded, background: getComputedStyle(document.body).backgroundColor });
  window.__AE_STROM_READY__ = true;
});
window.__RADIO_UI_READY__ = ready.then(() => { window.__RADIO_UI_READY__ = true; }).catch(error => {
  window.__RADIO_BOOT_ERROR__ = String(error);
  console.error(error);
  if (window.__TAURI__) bridge.shutdown().catch(console.error);
});

if (window.__RADIO_SMOKE_AUTORUN__) import('./native-smoke.js').then(({ runNativeSmoke }) => ready.then(() => runNativeSmoke(ui, bridge)));
