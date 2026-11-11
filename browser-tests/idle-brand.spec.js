import {test,expect} from '@playwright/test';
const brand='-= Æ Strøm =-';
const text=page=>page.locator('.radio-ticker .dui-scrolling-label__text').first();
const ticker=page=>page.locator('.radio-ticker');
test.beforeEach(async({page})=>{await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);});

test('idle branding is static centered accessible standard ScrollingLabel text',async({page})=>{
 await expect(text(page)).toHaveText(brand);await expect(ticker(page)).toHaveClass(/radio-ticker--idle/);await expect(ticker(page)).toHaveAttribute('aria-label',new RegExp('Æ Strøm'));
 const m=await ticker(page).evaluate(e=>{const a=e.getBoundingClientRect(),b=e.querySelector('.dui-scrolling-label__text').getBoundingClientRect();return{error:Math.abs((a.left+a.right-b.left-b.right)/2),running:e.getAnimations({subtree:true}).filter(a=>a.playState==='running').length,color:getComputedStyle(e).color,textAlign:getComputedStyle(e).textAlign};});
 expect(m.error).toBeLessThan(.75);expect(m.running).toBe(0);expect(m.textAlign).toBe('center');expect(m.color).not.toBe('rgba(0, 0, 0, 0)');
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions)).toEqual([]);expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:36});
 await page.evaluate(()=>window.radioPreview.bridge.update({fullTitle:'Old metadata must not leak into idle',displayTitle:'Old metadata must not leak into idle'}));await expect(text(page)).toHaveText(brand);
});

test('Play Stop and stopped station selection preserve the idle transition contract',async({page})=>{
 await page.keyboard.press('2');expect(await page.evaluate(()=>window.radioPreview.ui.getState().snapshot.activeSlot)).toBe(1);await expect(text(page)).toHaveText(brand);
 await page.getByRole('button',{name:'Play',exact:true}).click();await expect(page.getByRole('button',{name:'Stop',exact:true})).toBeVisible();await expect(ticker(page)).not.toHaveClass(/radio-ticker--idle/);
 const title='Synthetic artist — a long track title '.repeat(20);await page.evaluate(title=>window.radioPreview.bridge.update({status:'playing',requestedPlaying:true,error:null,fullTitle:title,displayTitle:title}),title);
 await expect(text(page)).toHaveText(title.slice(0,240));await expect(ticker(page)).toHaveCSS('text-align','start');
 await expect.poll(()=>ticker(page).evaluate(e=>e.getAnimations({subtree:true}).some(a=>a.playState==='running'))).toBe(true);
 await page.keyboard.press('ControlOrMeta+c');expect(await page.evaluate(()=>window.radioPreview.bridge.copies.at(-1))).toBe(title);
 await page.getByRole('button',{name:'Stop',exact:true}).click();await expect(text(page)).toHaveText(brand);expect(await ticker(page).evaluate(e=>e.getAnimations({subtree:true}).some(a=>a.playState==='running'))).toBe(false);
});

test('connecting reconnecting and errors take precedence over branding',async({page})=>{
 for(const status of ['connecting','reconnecting']){const label=status==='connecting'?'Connecting...':'Reconnecting...';await page.evaluate(({status,label})=>window.radioPreview.bridge.update({status,requestedPlaying:true,error:null,fullTitle:label,displayTitle:label}),{status,label});await expect(text(page)).toHaveText(label);await expect(ticker(page)).not.toHaveClass(/radio-ticker--idle/);}
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'stopped',requestedPlaying:false,error:'No stream URL for the selected slot',fullTitle:'Stopped',displayTitle:'Stopped'}));await expect(text(page)).toHaveText('No stream URL for the selected slot');await expect(ticker(page)).not.toHaveClass(/radio-ticker--idle/);
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'error',error:'Synthetic decoder failure'}));await expect(text(page)).toHaveText('Synthetic decoder failure');
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'stopped',requestedPlaying:false,error:null}));await expect(text(page)).toHaveText(brand);
});

test('compact expand and narrow ticker do not grow the window or animate idle',async({page})=>{
 await page.getByRole('button',{name:'Station settings',exact:true}).click();const input=page.locator('[data-station-slot="5"]');await input.fill('https://synthetic.invalid/draft');
 await page.getByRole('button',{name:'Collapse player',exact:true}).click();await expect(ticker(page)).toBeHidden();expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:102,height:36});
 await page.getByRole('button',{name:'Expand player',exact:true}).click();await expect(text(page)).toHaveText(brand);await expect(input).toHaveValue('https://synthetic.invalid/draft');await expect(page.locator('.settings-panel')).toBeVisible();
 await ticker(page).evaluate(e=>{e.style.flex='0 0 70px';e.style.width='70px';});
 const m=await ticker(page).evaluate(e=>{const a=e.getBoundingClientRect(),b=e.querySelector('.dui-scrolling-label__text').getBoundingClientRect();return{width:a.width,error:Math.abs((a.left+a.right-b.left-b.right)/2),overflow:getComputedStyle(e).overflow,running:e.getAnimations({subtree:true}).some(a=>a.playState==='running')};});
 expect(m.width).toBe(70);expect(m.error).toBeLessThan(.75);expect(m.overflow).toBe('hidden');expect(m.running).toBe(false);
});
