import {test,expect} from '@playwright/test';
test.beforeEach(async({page})=>{await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__);await page.evaluate(()=>document.fonts.ready);});

test('channel has uniform compact geometry across all260 bank/slot combinations including W and M',async({page})=>{
 const metrics=await page.evaluate(()=>{
  const results=[];for(let bank=0;bank<26;bank++)for(let slot=0;slot<10;slot++){
   window.radioPreview.bridge.update({activeBank:bank,activeSlot:slot});
   const e=document.querySelector('.channel-indicator'),b=e.firstElementChild,c=e.querySelector('.channel-slot'),r=e.getBoundingClientRect(),br=b.getBoundingClientRect(),cr=c.getBoundingClientRect(),range=document.createRange();range.selectNodeContents(b);const ink=range.getBoundingClientRect(),style=getComputedStyle(e);
   results.push({label:e.getAttribute('aria-label'),width:r.width,height:r.height,bankWidth:br.width,circleWidth:cr.width,circleHeight:cr.height,gap:cr.x-br.right,right:r.right-cr.right,letterFits:ink.x>=br.x&&ink.right<=br.right,centers:Math.abs(cr.y+cr.height/2-r.y-r.height/2),font:getComputedStyle(b).fontSize,circleFont:getComputedStyle(c).fontSize,padding:[style.paddingLeft,style.paddingRight],border:style.borderTopColor});
  }return results;
 });
 expect(metrics).toHaveLength(260);for(const [i,m] of metrics.entries()){expect(m.label).toBe(`Bank ${String.fromCharCode(65+Math.floor(i/10))}, station ${(i%10+1)%10}`);expect(m.width).toBe(40);expect(m.height).toBe(26);expect(m.bankWidth).toBe(14);expect(m.circleWidth).toBe(17);expect(m.circleHeight).toBe(17);expect(m.gap).toBe(2);expect(m.right).toBe(3);expect(m.letterFits).toBe(true);expect(m.centers).toBeLessThan(.01);expect(m.font).toBe('12px');expect(m.circleFont).toBe('12px');expect(m.padding).toEqual(['3px','2px']);expect(m.border).toBe('rgb(255, 255, 255)');}
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions)).toEqual([]);expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:36});
});

test('nine freed pixels return to ticker, panel dimensions and channel drag routing are preserved',async({page})=>{
 const read=()=>page.evaluate(()=>Object.fromEntries(['.radio-ticker','.radio-display','.radio-volume','.player-strip'].map(sel=>{const b=document.querySelector(sel).getBoundingClientRect();return[sel,{x:b.x,y:b.y,width:b.width,height:b.height}];})));
 const tight=await read();
 await page.locator('.channel-indicator').evaluate(e=>{e.style.cssText='width:49px;min-width:49px;max-width:49px;flex:0 0 49px;padding:2px 4px;gap:4px;justify-content:space-around';e.firstElementChild.style.cssText='width:auto;flex:0 1 auto;text-align:start';});
 const previous=await read();expect(tight['.radio-ticker'].width-previous['.radio-ticker'].width).toBe(9);for(const sel of ['.radio-display','.radio-volume','.player-strip'])expect(tight[sel]).toEqual(previous[sel]);
 await page.locator('.channel-indicator').evaluate(e=>{e.removeAttribute('style');e.firstElementChild.removeAttribute('style');});
 await expect(page.locator('.radio-display')).toHaveCSS('background-color','rgb(0, 61, 102)');
 for(const selector of ['.channel-indicator>span:first-child','.channel-slot']){const b=await page.locator(selector).boundingBox();await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.down();await page.mouse.move(b.x+b.width/2+10,b.y+b.height/2,{steps:4});await page.mouse.up();}
 expect(await page.evaluate(()=>window.radioPreview.bridge.dragCount)).toBe(2);expect(await page.evaluate(()=>window.radioPreview.bridge.actions)).toEqual([]);
 await page.getByRole('button',{name:'Station settings',exact:true}).click();expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:258});
 await page.getByRole('button',{name:'Equalizer',exact:true}).click();expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:257});
 await page.getByRole('button',{name:'Collapse player',exact:true}).click();expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:102,height:36});await expect(page.locator('.channel-indicator')).toBeHidden();await page.getByRole('button',{name:'Expand player',exact:true}).click();await expect(page.locator('.channel-indicator')).toHaveCSS('width','40px');
});
