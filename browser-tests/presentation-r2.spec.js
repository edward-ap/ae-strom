import { test, expect } from '@playwright/test';
test.beforeEach(async ({page})=>{await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);});

test('continuous blue display, black chrome and five narrower painted bands',async({page})=>{
 await expect(page.locator('.radio-display')).toHaveCSS('background-color','rgb(0, 61, 102)');
 await expect(page.locator('.radio-owner')).toHaveCSS('background-color','rgb(0, 0, 0)');
 await expect(page.locator('.channel-indicator')).toHaveCSS('border-top-color','rgb(255, 255, 255)');
 const buttons=await page.locator('.player-strip button').evaluateAll(es=>es.map(e=>getComputedStyle(e).borderTopColor));expect(buttons.every(c=>c==='rgba(0, 0, 0, 0)')).toBe(true);
 await expect.poll(()=>page.locator('.radio-spectrum canvas').evaluate(e=>Array.from(e.getContext('2d').getImageData(0,0,1,1).data))).toEqual([0,61,102,255]);
 const bands=await page.locator('.radio-spectrum canvas').evaluate(e=>{
  const dpr=e.width/e.clientWidth,data=e.getContext('2d').getImageData(0,Math.floor(e.height-2*dpr),e.width,1).data;
  const widths=[];let length=0;for(let x=0;x<e.width;x++){if(data[x*4]>20)length++;else if(length){widths.push(length/dpr);length=0;}}if(length)widths.push(length/dpr);
  return{widths,rect:e.getBoundingClientRect().toJSON()};
 });expect(bands.rect).toMatchObject({width:42,height:24});expect(bands.widths).toHaveLength(5);expect(Math.max(...bands.widths)).toBeLessThan(7);
 const points=await page.evaluate(()=>{const s=document.querySelector('.radio-spectrum').getBoundingClientRect(),t=document.querySelector('.radio-ticker').getBoundingClientRect(),c=document.querySelector('.channel-indicator').getBoundingClientRect();return [[(s.right+t.left)/2,s.y+12],[(t.right+c.left)/2,t.y+12]];});
 const png=await page.screenshot();
 const colors=await page.evaluate(async({png,points})=>{const bitmap=await createImageBitmap(new Blob([Uint8Array.from(atob(png),c=>c.charCodeAt(0))],{type:'image/png'}));const c=document.createElement('canvas');c.width=bitmap.width;c.height=bitmap.height;const ctx=c.getContext('2d');ctx.drawImage(bitmap,0,0);return points.map(([x,y])=>Array.from(ctx.getImageData(Math.floor(x*devicePixelRatio),Math.floor(y*devicePixelRatio),1,1).data));},{png:png.toString('base64'),points});
 expect(colors).toEqual([[0,61,102,255],[0,61,102,255]]);
 // The new common background retains the old strip-gap drag surface.
 const [x,y]=points[0];await page.mouse.move(x,y);await page.mouse.down();await page.mouse.move(x+9,y,{steps:4});await page.mouse.up();
 expect(await page.evaluate(()=>window.radioPreview.bridge.dragCount)).toBe(1);
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions.length)).toBe(0);

});

test('public grip volume keeps pointer and keyboard behavior without playback actions',async({page})=>{
 const volume=page.locator('.radio-volume input[type=range]');await expect(page.locator('.radio-volume')).toHaveAttribute('data-thumb-shape','grip');
 await volume.focus();await volume.press('Home');await expect(volume).toHaveValue('0');await volume.press('ArrowRight');expect(Number(await volume.inputValue())).toBeGreaterThan(0);
 await volume.press('End');await expect(volume).toHaveValue('100');await volume.press('ArrowLeft');expect(Number(await volume.inputValue())).toBeLessThan(100);
 const box=await volume.boundingBox();await page.mouse.click(box.x+box.width/2,box.y+box.height/2);expect(Number(await volume.inputValue())).toBeGreaterThanOrEqual(40);expect(Number(await volume.inputValue())).toBeLessThanOrEqual(60);
 await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();await page.mouse.move(box.x+box.width*.8,box.y+box.height/2,{steps:12});await page.mouse.up();expect(Number(await volume.inputValue())).toBeGreaterThan(65);
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions.every(a=>a.type==='setVolume'))).toBe(true);
 expect(await page.evaluate(()=>window.radioPreview.ui.getState().snapshot.requestedPlaying)).toBe(false);
});
