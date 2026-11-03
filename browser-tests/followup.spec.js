import {test,expect} from '@playwright/test';

test('startup readiness waits for bundled font and settled host layout without a timer',async({page})=>{
 let release;const gate=new Promise(resolve=>release=resolve);await page.route('**/*.woff2',async route=>{await gate;await route.continue();});
 await page.goto('/',{waitUntil:'domcontentloaded'});await page.waitForFunction(()=>Boolean(window.radioPreview));
 expect(await page.evaluate(()=>window.__RADIO_UI_READY__===true)).toBe(false);expect(await page.evaluate(()=>window.radioPreview.bridge.readySignals.length)).toBe(0);
 release();await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);
 expect(await page.evaluate(()=>window.radioPreview.bridge.readySignals)).toEqual([{width:725,height:36,fontLoaded:true,background:'rgb(0, 0, 0)'}]);
 expect(await page.evaluate(()=>window.__AE_STROM_READY__)).toBe(true);expect(await page.evaluate(()=>window.radioPreview.bridge.layouts.at(-1))).toEqual({compact:false,width:725,height:36});
});

test('standard Noto Sans12 is loaded from pinned local WOFF2 throughout the player',async({page},testInfo)=>{
 const fontResponse=page.waitForResponse(r=>r.url().endsWith('/fonts/NotoSans-variable.woff2'));
 await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);expect((await fontResponse).status()).toBe(200);
 await page.getByRole('button',{name:'Station settings',exact:true}).click();
 const selectors=['body','.radio-owner','.radio-ticker','.station-row input','.station-row select','.player-strip button','.bank-strip button','.column-label','.channel-slot'];
 const fonts=await page.evaluate(selectors=>selectors.map(selector=>{const style=getComputedStyle(document.querySelector(selector));return{selector,family:style.fontFamily,size:style.fontSize};}),selectors);
 for(const font of fonts){expect(font.family).toBe('"Noto Sans", "Segoe UI", Arial, sans-serif');expect(font.size,font.selector).toBe('12px');}
 expect(await page.evaluate(()=>[...document.fonts].some(f=>f.family.includes('Noto Sans')&&f.status==='loaded'))).toBe(true);expect(await page.evaluate(()=>document.fonts.check('12px "Noto Sans"','Æ Ø æ ø Живое радио'))).toBe(true);
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'playing',requestedPlaying:true,fullTitle:'Æ Strøm — Живое радио',displayTitle:'Æ Strøm — Живое радио'}));await page.locator('[data-station-slot="5"]').fill('https://synthetic.invalid/Æ-Strøm/музыка');await page.screenshot({path:testInfo.outputPath('standard-font-characters.png')});
 await page.getByRole('button',{name:'Equalizer',exact:true}).click();for(const selector of ['.dui-trackbar__label','.dui-trackbar__value','.eq-grid-label']){const s=await page.locator('.eq-panel '+selector).first().evaluate(e=>({family:getComputedStyle(e).fontFamily,size:getComputedStyle(e).fontSize}));expect(s.family).toContain('Noto Sans');expect(s.size).toBe('12px');}
 expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:257});
});

test('empty station and genuine error feedback never protrude below chrome or open a panel',async({page})=>{
 await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);
 await page.evaluate(()=>{
  const b=window.radioPreview.bridge,s=window.radioPreview.ui.getState().snapshot,dispatch=b.dispatch;
  b.update({stations:s.stations.map((station,i)=>i>=10&&i<20?{...station,url:'',name:''}:station)});
  b.dispatch=async action=>{if(action.type==='selectStation'&&action.bank===1){b.update({activeBank:1,activeSlot:action.slot,requestedPlaying:false,status:'stopped',fullTitle:'No stream URL',displayTitle:'No stream URL',error:'No stream URL for the selected slot'});throw Error('No stream URL for the selected slot');}return dispatch(action);};
 });
 await page.getByRole('button',{name:'Play',exact:true}).click();await page.keyboard.press('b');
 await expect(page.locator('.radio-ticker .dui-scrolling-label__text').first()).toHaveText('No stream URL for the selected slot');await expect(page.locator('.radio-status')).toHaveText('No stream URL for the selected slot');
 expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:36});await expect(page.locator('.settings-panel')).toBeHidden();await expect(page.locator('.eq-panel')).toBeHidden();
 const live=await page.locator('.radio-status').evaluate(e=>({rect:e.getBoundingClientRect().toJSON(),position:getComputedStyle(e).position,clip:getComputedStyle(e).clipPath,role:e.getAttribute('role'),ariaLive:e.getAttribute('aria-live')}));expect(live.position).toBe('absolute');expect(live.clip).toBe('inset(50%)');expect(live.rect.width).toBe(1);expect(live.rect.height).toBe(1);expect(live.role).toBe('status');expect(live.ariaLive).toBe('polite');
 await page.locator('.radio-volume input').press('ArrowRight');await page.evaluate(()=>window.radioPreview.bridge.update({error:'Synthetic decoder failure',status:'error',requestedPlaying:false}));await expect(page.locator('.radio-ticker .dui-scrolling-label__text').first()).toHaveText('Synthetic decoder failure');await expect(page.getByRole('button',{name:'Play',exact:true})).toHaveAttribute('aria-description','Synthetic decoder failure');
 expect(await page.evaluate(()=>window.radioPreview.bridge.layouts.every(x=>x.height===36))).toBe(true);
});
