import {test,expect} from '@playwright/test';
test.beforeEach(async({page})=>{await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);});

test('Settings paired headers align, bottom banks are equal framed squares, slots are circles',async({page})=>{
 await page.getByRole('button',{name:'Station settings',exact:true}).click();
 await expect(page.locator('.bank-info,.settings-panel .panel-hint')).toHaveCount(0);
 await expect(page.locator('.settings-panel')).not.toContainText('Editing bank');await expect(page.locator('.settings-panel')).not.toContainText('URLs save');
 const result=await page.evaluate(()=>{
  const box=e=>{const b=e.getBoundingClientRect();return{x:b.x,y:b.y,width:b.width,height:b.height,bottom:b.bottom}};
  const headers=[...document.querySelectorAll('.station-header')];const rows=[...document.querySelectorAll('.station-row')];
  return{headerLabels:headers.map(e=>[...e.querySelectorAll('.column-label')].map(c=>c.textContent)),headers:headers.map(box),
   tracks:headers.map((h,i)=>({header:[...h.querySelector('.dui-form__cols').children].map(box),row:[...rows[i*5].querySelector('.dui-form__cols').children].map(box)})),
   bankRow:box(document.querySelector('.bank-strip')),columns:box(document.querySelector('.station-columns')),owner:box(document.querySelector('.radio-owner')),
   banks:[...document.querySelectorAll('.bank-strip button')].map(e=>({text:e.textContent,box:box(e),border:getComputedStyle(e).borderTopColor,radius:getComputedStyle(e).borderTopLeftRadius})),
   slots:[...document.querySelectorAll('.station-row button')].map(e=>({box:box(e),border:getComputedStyle(e).borderTopColor,radius:getComputedStyle(e).borderTopLeftRadius})),
   toolbar:[...document.querySelectorAll('.player-strip button')].map(e=>getComputedStyle(e).borderTopColor)};
 });
 expect(result.headerLabels).toEqual([['Key','Stream URL','EQ Preset'],['Key','Stream URL','EQ Preset']]);
 expect(Math.abs(result.headers[0].y-result.headers[1].y)).toBeLessThan(.75);
 for(const pair of result.tracks)for(let i=0;i<3;i++){expect(Math.abs(pair.header[i].x-pair.row[i].x)).toBeLessThan(.75);expect(Math.abs(pair.header[i].width-pair.row[i].width)).toBeLessThan(.75);}
 expect(result.banks).toHaveLength(26);for(const b of result.banks){expect(b.box).toMatchObject({width:24,height:24});expect(b.border).toBe('rgb(114, 119, 128)');expect(b.radius).toBe('4px');}
 expect(result.slots).toHaveLength(10);for(const s of result.slots){expect(s.box).toMatchObject({width:26,height:26});expect(s.border).toBe('rgb(114, 119, 128)');expect(s.radius).toBe('50%');}
 expect(result.toolbar.every(c=>c==='rgba(0, 0, 0, 0)')).toBe(true);
 expect(result.bankRow.y-result.columns.bottom).toBe(8);expect(result.owner.bottom-result.bankRow.bottom).toBe(8);
 expect(result.owner).toMatchObject({width:725,height:258});
 const a=page.locator('[data-station-slot="5"]'),valueA='https://synthetic.invalid/draft-A?q=%22';await a.fill(valueA);
 await page.getByRole('button',{name:'Browse bank Z',exact:true}).click();await expect(page.getByRole('button',{name:'Browse bank Z',exact:true})).toHaveAttribute('aria-pressed','true');
 const valueZ='https://synthetic.invalid/draft-Z?q=%D0%96';await a.fill(valueZ);await page.getByRole('button',{name:'Browse bank A',exact:true}).click();await expect(a).toHaveValue(valueA);
 await page.getByRole('button',{name:'Browse bank Z',exact:true}).click();await expect(a).toHaveValue(valueZ);
 expect(await page.evaluate(()=>window.radioPreview.ui.getState().snapshot.activeBank)).toBe(0);
 expect(await page.evaluate(()=>window.radioPreview.ui.getState().browsedBank)).toBe(25);
 await expect(page.locator('.channel-indicator')).toHaveAttribute('aria-label','Bank A, station 1');
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions.some(a=>['setPlaying','selectStation'].includes(a.type)))).toBe(false);
});

test('drag routes once for chrome, ticker and nested channel; interactive and Settings gestures stay local',async({page})=>{
 const count=()=>page.evaluate(()=>window.radioPreview.bridge.dragCount);
 const gesture=async point=>{await page.mouse.move(point.x,point.y);await page.mouse.down();await page.mouse.move(point.x+9,point.y,{steps:3});await page.mouse.move(point.x+18,point.y,{steps:3});await page.mouse.up();};
 const strip=await page.locator('.player-strip').boundingBox();let expected=0;
 await gesture({x:strip.x+2,y:strip.y+2});expect(await count()).toBe(++expected);
 for(const selector of ['.radio-ticker','.channel-indicator > span:first-child','.channel-slot','.radio-spectrum canvas']){const b=await page.locator(selector).boundingBox();await gesture({x:b.x+b.width/2,y:b.y+b.height/2});expect(await count()).toBe(++expected);}
 for(const name of ['collapse','play','mute','settings','equalizer','close']){const b=await page.locator(`[data-control="${name}"]`).boundingBox();await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.down();await page.mouse.move(b.x+b.width/2,b.y+70,{steps:5});await page.mouse.up();expect(await count()).toBe(expected);}
 const volume=await page.locator('.radio-volume input').boundingBox();await gesture({x:volume.x+volume.width/2,y:volume.y+volume.height/2});expect(await count()).toBe(expected);
 await page.getByRole('button',{name:'Station settings',exact:true}).click();const field=page.locator('[data-station-slot="5"]');await field.fill('https://synthetic.invalid/'+ 'selection/'.repeat(20));const b=await field.boundingBox();await gesture({x:b.x+10,y:b.y+b.height/2});expect(await count()).toBe(expected);
 const panel=await page.locator('.settings-panel').boundingBox();await gesture({x:panel.x+2,y:panel.y+2});expect(await count()).toBe(expected);
 await page.evaluate(()=>window.radioPreview.ui.setCompact(true));const spectrum=await page.locator('.radio-spectrum').boundingBox();await gesture({x:spectrum.x+5,y:spectrum.y+8});expect(await count()).toBe(++expected);
 await page.evaluate(()=>window.radioPreview.ui.setCompact(false));
 const full='Synthetic full title '.repeat(30);await page.evaluate(full=>window.radioPreview.bridge.update({fullTitle:full,displayTitle:full.slice(0,240)}),full);
 await page.locator('.radio-ticker').click();await page.keyboard.press('ControlOrMeta+C');expect(await page.evaluate(()=>window.radioPreview.bridge.copies.at(-1))).toBe(full);
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions.some(a=>['setPlaying','selectStation'].includes(a.type)))).toBe(false);
});

test('drag cancellation, secondary button, popup guards and repeat moves do not leak calls',async({page})=>{
 const result=await page.evaluate(()=>{
  const ticker=document.querySelector('.radio-ticker'),strip=document.querySelector('.player-strip'),bridge=window.radioPreview.bridge;
  const emit=(target,type,extra={})=>target.dispatchEvent(new PointerEvent(type,{bubbles:true,button:0,buttons:1,pointerId:9,clientX:120,clientY:18,...extra}));
  emit(ticker,'pointerdown');emit(document,'pointermove',{pointerId:10,clientX:140});emit(document,'pointercancel');emit(document,'pointermove',{clientX:150});
  emit(ticker,'pointerdown',{button:2,buttons:2});emit(document,'pointermove',{buttons:2,clientX:160});emit(document,'pointerup');
  const popup=document.createElement('div');popup.setAttribute('role','listbox');strip.append(popup);emit(popup,'pointerdown');emit(document,'pointermove',{clientX:150});emit(document,'pointerup');popup.remove();
  const blocked=bridge.dragCount;
  emit(ticker,'pointerdown');emit(window,'blur');emit(document,'pointermove',{clientX:160});emit(document,'pointerup');
  const afterBlur=bridge.dragCount;
  emit(ticker,'pointerdown');for(const x of [126,135,150])emit(document,'pointermove',{clientX:x});emit(document,'pointerup');
  return{blocked,afterBlur,final:bridge.dragCount,tauriRegions:document.querySelectorAll('[data-tauri-drag-region]').length};
 });expect(result).toEqual({blocked:0,afterBlur:0,final:1,tauriRegions:0});
});
