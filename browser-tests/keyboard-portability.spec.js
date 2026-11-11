import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
 await page.goto('/');
 await page.waitForFunction(() => window.__RADIO_UI_READY__ === true);
});

test('physical Cmd+C and Ctrl+C copy metadata once without selecting a bank', async ({ page }) => {
 const result = await page.evaluate(() => {
  const bridge = window.radioPreview.bridge;
  bridge.update({ fullTitle: 'Synthetic full title for clipboard' });
  const actions = bridge.actions.length;
  const prevented = [ {metaKey:true}, {ctrlKey:true} ].map(modifier => {
   // Physical C must work even when the active layout emits a different character.
   const event = new KeyboardEvent('keydown', {bubbles:true,cancelable:true,code:'KeyC',key:'с',...modifier});
   document.body.dispatchEvent(event);
   return event.defaultPrevented;
  });
  return { prevented, copies:bridge.copies, actions:bridge.actions.length-actions };
 });
 expect(result).toEqual({prevented:[true,true],copies:['Synthetic full title for clipboard','Synthetic full title for clipboard'],actions:0});
});

test('copy rejects extra modifiers, repeats, composition and already-handled events', async ({ page }) => {
 const result = await page.evaluate(() => {
  const bridge = window.radioPreview.bridge;
  const before = {copies:bridge.copies.length,actions:bridge.actions.length};
  const cases = [];
  for (const modifier of [{metaKey:true},{ctrlKey:true}]) {
   for (const guard of [{altKey:true},{shiftKey:true},{repeat:true},{isComposing:true},{handled:true}]) cases.push({...modifier,...guard});
  }
  cases.push({ctrlKey:true,metaKey:true});
  const prevented = cases.map(({handled,...details}) => {
   const event = new KeyboardEvent('keydown',{bubbles:true,cancelable:true,code:'KeyC',key:'c',...details});
   if (handled) event.preventDefault();
   document.body.dispatchEvent(event);
   return event.defaultPrevented === !!handled;
  });
  for (const modifier of [{metaKey:true},{ctrlKey:true},{altKey:true}]) {
   document.body.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,cancelable:true,code:'KeyQ',key:'q',...modifier}));
  }
  return {unchanged:before.copies===bridge.copies.length && before.actions===bridge.actions.length,prevented};
 });
 expect(result.unchanged).toBe(true);
 expect(result.prevented.every(Boolean)).toBe(true);
});

test('Cmd/Ctrl copy stays native in editor and accessibility control targets', async ({ page }) => {
 const results = await page.evaluate(() => {
  const bridge = window.radioPreview.bridge;
  const before = {copies:bridge.copies.length,actions:bridge.actions.length};
  const results = [];
  for (const markup of ['<input>','<textarea></textarea>','<select><option>Text</option></select>','<div contenteditable="true"><span>Text</span></div>','<div role="combobox"><span>Text</span></div>','<div role="slider"></div>','<div role="listbox"></div>']) {
   const host = document.createElement('div');host.innerHTML=markup;document.body.append(host);
   const target=host.querySelector('span') || host.firstElementChild;
   for (const modifier of [{metaKey:true},{ctrlKey:true}]) {
    const event=new KeyboardEvent('keydown',{bubbles:true,cancelable:true,code:'KeyC',key:'c',...modifier});
    target.dispatchEvent(event);results.push(!event.defaultPrevented);
   }
   host.remove();
  }
  return {results,unchanged:before.copies===bridge.copies.length && before.actions===bridge.actions.length};
 });
 expect(results.unchanged).toBe(true);
 expect(results.results).toHaveLength(14);
 expect(results.results.every(Boolean)).toBe(true);
});
