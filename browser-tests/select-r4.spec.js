import {test,expect} from '@playwright/test';
const block='████████████████████████████';
test.beforeEach(async({page})=>{await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);await page.getByRole('button',{name:'Station settings',exact:true}).click();});
async function choices(page){await page.evaluate(block=>{const s=window.radioPreview.ui.getState().snapshot;const names=['Flat','Acoustic','Synthetic equalizer preset WWW iii',block];window.radioPreview.bridge.update({eqPresets:names.map(name=>({name,preamp:0,bands:Array(10).fill(0),builtin:name==='Flat'})),stations:s.stations.map((x,i)=>i===0||i===5?{...x,eq:'Acoustic'}:x)});},block);}

test('native EQ square area equals field height with one text reserve and popup keyboard behavior',async({page})=>{
 await choices(page);
 const all=await page.locator('.station-row select').evaluateAll(es=>es.map(e=>{const s=getComputedStyle(e),a=getComputedStyle(e.parentElement,'::after'),b=e.getBoundingClientRect();return{width:b.width,height:b.height,font:s.fontSize,paddingEnd:s.paddingInlineEnd,appearance:s.appearance,squareWidth:a.width,squareHeight:a.height,pointerEvents:a.pointerEvents,custom:[...e.children].some(c=>c.tagName!=='OPTION')};}));
 for(const m of all)expect(m).toEqual({width:110,height:24,font:'12px',paddingEnd:'25px',appearance:'none',squareWidth:'24px',squareHeight:'24px',pointerEvents:'none',custom:false});
 for(const slot of [1,6]){
  const field=page.getByRole('combobox',{name:`Station ${slot} equalizer`,exact:true});await field.focus();await field.press('End');await expect.poll(()=>field.evaluate(e=>e.selectedOptions[0].text)).toBe(block);
  const b=await field.boundingBox();await page.mouse.click(b.x+b.width-6,b.y+b.height/2);await page.keyboard.press('Home');await page.keyboard.press('ArrowDown');await page.keyboard.press('Enter');await expect.poll(()=>field.evaluate(e=>e.selectedOptions[0].text)).toBe('Flat');await expect(field).toBeFocused();await page.mouse.click(850,500);
  await field.selectOption({label:block});await field.evaluate(e=>{e.style.color='rgb(255,0,197)';for(const o of e.options)o.style.color='rgb(255,0,197)';});
  const png=await field.screenshot();const paint=await page.evaluate(async png=>{const b=await createImageBitmap(new Blob([Uint8Array.from(atob(png),c=>c.charCodeAt(0))],{type:'image/png'}));const c=document.createElement('canvas');c.width=b.width;c.height=b.height;const ctx=c.getContext('2d');ctx.drawImage(b,0,0);const a=ctx.getImageData(0,0,c.width,c.height).data,text=[],arrow=[];for(let x=0;x<c.width;x++){let pink=0,white=0;for(let y=3*devicePixelRatio;y<c.height-3*devicePixelRatio;y++){const i=(Math.floor(y)*c.width+x)*4;if(a[i]>190&&a[i+1]<90&&a[i+2]>130)pink++;if(a[i]>190&&a[i+1]>190&&a[i+2]>190&&Math.max(a[i],a[i+1],a[i+2])-Math.min(a[i],a[i+1],a[i+2])<20)white++;}if(pink>=8*devicePixelRatio)text.push(x);if(white&&x>86*devicePixelRatio)arrow.push(x);}return{textEnd:(Math.max(...text)+1)/devicePixelRatio,textWidth:(Math.max(...text)-Math.min(...text)+1)/devicePixelRatio,arrowCenter:(Math.min(...arrow)+Math.max(...arrow)+1)/2/devicePixelRatio,arrowWidth:(Math.max(...arrow)-Math.min(...arrow)+1)/devicePixelRatio};},png.toString('base64'));
  expect(paint.textEnd).toBeGreaterThanOrEqual(82);expect(paint.textEnd).toBeLessThanOrEqual(85);expect(Math.abs(paint.arrowCenter-98)).toBeLessThanOrEqual(1.1);expect(paint.arrowWidth).toBeGreaterThanOrEqual(8);expect(paint.arrowWidth).toBeLessThanOrEqual(11);
  const geometry=await field.evaluate(e=>{const b=e.getBoundingClientRect(),s=getComputedStyle(e),a=getComputedStyle(e.parentElement,'::after');return{area:b.width-parseFloat(a.width),textBoundary:b.width-parseFloat(s.borderRightWidth)-parseFloat(s.paddingRight)};});expect(geometry.area-geometry.textBoundary).toBe(2);
 }
 expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:258});expect(await page.evaluate(()=>window.radioPreview.bridge.dragCount)).toBe(0);expect(await page.evaluate(()=>window.radioPreview.bridge.actions.some(a=>['setPlaying','selectStation'].includes(a.type)))).toBe(false);
});

test('selected slot and bank are ticker blue with white text and gray frames only',async({page})=>{
 const bank=page.getByRole('button',{name:'Browse bank A',exact:true}),slot=page.locator('.station-row button[aria-checked="true"]');
 for(const field of [bank,slot]){await expect(field).toHaveCSS('background-color','rgb(0, 61, 102)');await expect(field).toHaveCSS('color','rgb(255, 255, 255)');await expect(field).toHaveCSS('border-top-color','rgb(114, 119, 128)');await field.hover();await expect(field).toHaveCSS('background-color','rgb(0, 61, 102)');await field.focus();await field.press('Tab');await page.keyboard.press('Shift+Tab');await expect(field).toBeFocused();expect(await field.evaluate(e=>parseFloat(getComputedStyle(e).outlineWidth))).toBeGreaterThanOrEqual(2);expect(await field.evaluate(e=>getComputedStyle(e).outlineStyle)).not.toBe('none');}
 await expect(page.locator('.station-row select').first()).toHaveCSS('background-color','rgb(41, 44, 51)');
 await page.mouse.move(850,500);await expect(page.locator('[data-control="settings"]')).toHaveCSS('background-color','rgb(54, 80, 63)');
 await expect(page.getByRole('button',{name:'Browse bank B',exact:true})).toHaveCSS('background-color','rgb(48, 50, 58)');
 await page.getByRole('button',{name:'Browse bank Z',exact:true}).click();await expect(page.getByRole('button',{name:'Browse bank Z',exact:true})).toHaveCSS('background-color','rgb(0, 61, 102)');
 expect(await page.evaluate(()=>window.radioPreview.ui.getState().snapshot.activeBank)).toBe(0);await expect(page.locator('.station-row button[aria-checked="true"]')).toHaveCount(0);
});
