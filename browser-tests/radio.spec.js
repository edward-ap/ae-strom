import { test, expect } from '@playwright/test';
const urlFor = n => `https://synthetic.invalid/station-${n}/` + 'segment0123456789/'.repeat(42) + '?token=synthetic&last=finish';
const input = (page,slot) => page.locator(`[data-station-slot="${slot}"]`);
const state = page => page.evaluate(() => window.radioPreview.ui.getState());
const metrics = field => field.evaluate(e => ({value:e.value,start:e.selectionStart,end:e.selectionEnd,direction:e.selectionDirection,scroll:e.scrollLeft,rect:e.getBoundingClientRect().toJSON()}));
test.beforeEach(async ({page}) => { await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true); });

test('strip, compact ownership, drafts, focus, last panel and drag boundaries',async({page},testInfo)=>{
 await expect(page.locator('.player-strip')).toHaveCSS('height','36px');
 expect((await page.locator('.radio-owner').boundingBox()).width).toBe(725);
 await page.getByRole('button',{name:'Station settings',exact:true}).click();
 const editor=input(page,5);await editor.fill(urlFor(6));
 const before=await state(page);
 await page.getByRole('button',{name:'Collapse player',exact:true}).click();
 await expect(page.getByRole('button',{name:'Expand player',exact:true})).toBeVisible();
 await expect(editor).toBeHidden();
 expect((await page.locator('.player-strip').boundingBox()).width).toBeLessThan(180);
 expect((await state(page)).snapshot.generation).toBe(before.snapshot.generation);
 expect(await page.locator('.player-strip button:visible').count()).toBe(2);await expect(page.locator('.player-strip > :visible')).toHaveCount(3);await expect(page.locator('.channel-indicator')).toBeHidden();
 await page.screenshot({path:testInfo.outputPath('compact.png')});
 await page.setViewportSize({width:102,height:36});
 await page.getByRole('button',{name:'Expand player',exact:true}).press('Space');
 expect(await page.evaluate(()=>window.radioPreview.bridge.layouts.at(-1))).toEqual({compact:false,width:725,height:258});
 await page.setViewportSize({width:900,height:600});
 await expect(editor).toBeVisible();await expect(editor).toHaveValue(urlFor(6));
 expect((await state(page)).panel).toBe('settings');
 const spectrum=await page.locator('.radio-spectrum').boundingBox();
 await page.mouse.move(spectrum.x+10,spectrum.y+10);await page.mouse.down();await page.mouse.move(spectrum.x+22,spectrum.y+10);await page.mouse.up();
 expect(await page.evaluate(()=>window.radioPreview.bridge.dragCount)).toBe(1);
 await page.getByRole('button',{name:'Play',exact:true}).click();
 expect(await page.evaluate(()=>window.radioPreview.bridge.dragCount)).toBe(1);
 await page.getByRole('button',{name:'Stop',exact:true}).click();
 await page.getByRole('button',{name:'Collapse player',exact:true}).focus();await page.keyboard.press('Enter');await page.keyboard.press('Tab');
 await expect(page.getByRole('button',{name:'Play',exact:true})).toBeFocused();
 await page.keyboard.press('Shift+Tab');await expect(page.getByRole('button',{name:'Expand player',exact:true})).toBeFocused();
});

test('browsed bank is independent, physical key and editor/default guards',async({page})=>{
 await page.getByRole('button',{name:'Station settings',exact:true}).click();await page.getByRole('button',{name:'Browse bank Z',exact:true}).click();
 expect((await state(page)).snapshot.activeBank).toBe(0);expect((await state(page)).browsedBank).toBe(25);
 const editor=input(page,9);await editor.fill(urlFor(0));const calls=await page.evaluate(()=>window.radioPreview.bridge.actions.length);
 await editor.press('Control+A');await editor.press('Control+C');await editor.press('Control+X');await editor.press('Control+V');
 expect(await page.evaluate(()=>window.radioPreview.bridge.copies.length)).toBe(0);
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions.length)).toBe(calls);
 await page.getByRole('button',{name:'Station settings',exact:true}).focus();
 await page.keyboard.press('KeyB');await page.keyboard.press('Digit0');await page.keyboard.press('NumpadAdd');await page.keyboard.press('NumpadMultiply');await page.keyboard.press('Control+C');
 await expect.poll(async()=> (await state(page)).snapshot.activeBank).toBe(1);
 expect((await state(page)).snapshot.activeSlot).toBe(9);expect((await state(page)).snapshot.volume).toBe(85);expect((await state(page)).snapshot.muted).toBe(true);
 expect(await page.evaluate(()=>window.radioPreview.bridge.copies.length)).toBe(1);
 const after=await page.evaluate(()=>window.radioPreview.bridge.actions.length);
 await page.evaluate(()=>{for(const details of [{code:'KeyQ',key:'q',repeat:true},{code:'KeyQ',key:'q',isComposing:true},{code:'KeyQ',key:'q',altKey:true},{code:'KeyQ',key:'q',ctrlKey:true}])document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,...details}));});
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions.length)).toBe(after);
});

test('EQ assignment, 11 real vertical controls, custom save/delete',async({page},testInfo)=>{
 await page.getByRole('button',{name:'Equalizer',exact:true}).click();const sliders=page.locator('.eq-sliders input[type=range]');await expect(sliders).toHaveCount(11);
 await expect(sliders.first()).toHaveAttribute('aria-orientation','vertical');await expect(sliders.first()).toHaveAttribute('min','-15');await expect(sliders.first()).toHaveAttribute('max','15');
 for (const index of [0,10]) {
  const slider=sliders.nth(index),box=await slider.boundingBox();
  await page.mouse.click(box.x+box.width/2,box.y+2);await expect(slider).toHaveValue('15');
  await page.mouse.click(box.x+box.width/2,box.y+box.height-2);await expect(slider).toHaveValue('-15');
  await slider.press('ArrowUp');await expect(slider).toHaveValue('-14');
  await slider.press('End');await expect(slider).toHaveValue('15');await slider.press('Home');await expect(slider).toHaveValue('-15');
 }
 await expect(page.locator('.eq-sliders .dui-trackbar__tick-label')).toHaveCount(0);
 const sharedScale=await page.locator('.eq-grid-line').evaluateAll(es=>es.map(e=>({db:Number(e.dataset.eqDb),y:e.getBoundingClientRect().y})));
 expect(sharedScale.find(t=>t.db===15).y).toBeLessThan(sharedScale.find(t=>t.db===-15).y);
 await page.getByRole('textbox',{name:'Custom preset name',exact:true}).fill('Synthetic custom');await page.getByRole('button',{name:'Save',exact:true}).click();
 await expect.poll(async()=> (await state(page)).snapshot.eq.name).toBe('Synthetic custom');
 await page.getByRole('button',{name:'Delete',exact:true}).click();expect((await state(page)).snapshot.eqPresets.some(p=>p.name==='Synthetic custom')).toBe(false);
 await page.screenshot({path:testInfo.outputPath('equalizer.png')});
 await page.getByRole('button',{name:'Station settings',exact:true}).click();await page.getByRole('combobox',{name:'Station 6 equalizer',exact:true}).selectOption({label:'Rock'});
 expect((await state(page)).snapshot.stations[5].eq).toBe('Rock');
});

for(const slot of [0,5,6,7,8,9])test(`native URL selection and horizontal scroll slot ${slot+1}`,async({page},testInfo)=>{
 await page.getByRole('button',{name:'Play',exact:true}).click();await page.getByRole('button',{name:'Station settings',exact:true}).click();const field=input(page,slot);await field.fill(urlFor(slot));await field.press('Home');
 const bounds=await field.boundingBox();expect(bounds.width).toBeGreaterThan(200);
 await page.mouse.move(bounds.x+8,bounds.y+bounds.height/2);await page.mouse.down();await page.mouse.move(bounds.x+bounds.width+35,bounds.y+bounds.height/2,{steps:18});await page.waitForTimeout(250);
 const right=await metrics(field);expect(right.scroll).toBeGreaterThan(0);expect(right.end-right.start).toBeGreaterThan(10);
 await page.screenshot({path:testInfo.outputPath(`slot-${slot}-right-selection.png`)});
 await page.mouse.move(bounds.x-35,bounds.y+bounds.height/2,{steps:18});await page.waitForTimeout(250);const left=await metrics(field);await page.mouse.up();expect(left.scroll).toBeLessThan(right.scroll);
 await field.press('End');await field.press('Shift+Home');const reverse=await metrics(field);expect(reverse.start).toBe(0);expect(reverse.end).toBe(urlFor(slot).length);expect(reverse.direction).toBe('backward');
 await field.press('Home');await field.press('Shift+End');const forward=await metrics(field);expect(forward.start).toBe(0);expect(forward.end).toBe(urlFor(slot).length);expect(forward.direction).toBe('forward');
 await field.press('Control+Shift+ArrowLeft');await field.press('Shift+ArrowRight');
 const preserved=await metrics(field);
 await page.evaluate(()=>{window.radioPreview.bridge.update({fullTitle:'Long synthetic title '.repeat(30),displayTitle:'Long synthetic title '.repeat(30)});for(let i=0;i<20;i++)window.radioPreview.bridge.spectrum([.1,.3,.6,.9,.5]);});
 const after=await metrics(field);expect({value:after.value,start:after.start,end:after.end,scroll:after.scroll}).toEqual({value:preserved.value,start:preserved.start,end:preserved.end,scroll:preserved.scroll});
 // Painted selection sentinels, at device pixel ratio. Selection must stay inside its native edit viewport.
 await page.addStyleTag({content:'input[data-station-slot]::selection{background:rgb(255,0,197)!important;color:white!important}'});await field.press('Control+A');
 const png=await page.screenshot();const dpr=await page.evaluate(()=>devicePixelRatio);
 const paint=await page.evaluate(async({base64,bounds,dpr})=>{const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));const bitmap=await createImageBitmap(new Blob([bytes],{type:'image/png'}));const canvas=document.createElement('canvas');canvas.width=bitmap.width;canvas.height=bitmap.height;const ctx=canvas.getContext('2d');ctx.drawImage(bitmap,0,0);const data=ctx.getImageData(0,0,canvas.width,canvas.height).data;let inside=0,outside=0;for(let y=Math.ceil(bounds.y*dpr);y<Math.floor((bounds.y+bounds.height)*dpr);y++)for(let x=Math.max(0,Math.floor((bounds.x-30)*dpr));x<Math.min(canvas.width,Math.ceil((bounds.x+bounds.width+75)*dpr));x++){const i=(y*canvas.width+x)*4;if(data[i]===255&&data[i+1]===0&&data[i+2]===197){if(x>=bounds.x*dpr&&x<(bounds.x+bounds.width)*dpr)inside++;else outside++;}}return{inside,outside};},{base64:png.toString('base64'),bounds,dpr});
 expect(paint.inside).toBeGreaterThan(10);expect(paint.outside).toBe(0);
 await testInfo.attach('selection-metrics',{body:JSON.stringify({slot,dpr,right,left,reverse,forward,paint},null,2),contentType:'application/json'});
 await testInfo.attach('selected-url',{body:png,contentType:'image/png'});
 await field.press('Enter');expect((await state(page)).snapshot.stations[slot].url).toBe(urlFor(slot));
});

test('safe metadata text and teardown subscriptions',async({page})=>{
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'playing',requestedPlaying:true,fullTitle:'<img src=x onerror=alert(1)>'.repeat(20),displayTitle:'<img src=x onerror=alert(1)>'.repeat(20)}));
 expect(await page.locator('.radio-ticker').innerText()).toContain('<img');await expect(page.locator('.radio-ticker img')).toHaveCount(0);
 const displayed=await page.locator('.radio-ticker .dui-scrolling-label__text').first().innerText();expect(displayed.length).toBeLessThanOrEqual(240);
 await page.evaluate(()=>window.radioPreview.ui.destroy());expect(await page.locator('.radio-owner').count()).toBe(0);
 await page.evaluate(()=>{window.radioPreview.bridge.update({volume:12});window.radioPreview.bridge.spectrum([1,1,1,1,1]);});
});


test('error feedback remains inside36px chrome and accessible in compact mode',async({page})=>{
 await page.evaluate(()=>window.radioPreview.bridge.update({error:'Synthetic connection failure'}));
 await expect(page.locator('.radio-status')).toHaveText('Synthetic connection failure');await expect(page.locator('.radio-status')).toHaveCSS('clip-path','inset(50%)');
 expect((await page.locator('.radio-owner').boundingBox()).height).toBe(36);await expect(page.locator('.radio-ticker .dui-scrolling-label__text').first()).toHaveText('Synthetic connection failure');
 await page.getByRole('button',{name:'Collapse player',exact:true}).click();await expect(page.locator('.player-strip > :visible')).toHaveCount(3);await expect(page.getByRole('button',{name:'Play',exact:true})).toHaveAttribute('aria-description','Synthetic connection failure');
 expect(await page.evaluate(()=>window.radioPreview.bridge.layouts.at(-1).height)).toBe(36);
 await page.getByRole('button',{name:'Expand player',exact:true}).click();await expect(page.locator('.radio-ticker .dui-scrolling-label__text').first()).toHaveText('Synthetic connection failure');
 await page.evaluate(()=>window.radioPreview.bridge.update({error:null}));await expect(page.locator('.radio-status')).toHaveText('');expect((await page.locator('.radio-owner').boundingBox()).height).toBe(36);
});

test('spectrum clears unpowered/reconnecting state and rejects stale generation',async({page})=>{
 const canvas=page.locator('.radio-spectrum canvas');const pixels=()=>canvas.evaluate(e=>e.toDataURL());
 await page.waitForTimeout(60);const idle=await pixels();
 await page.evaluate(()=>{window.radioPreview.bridge.freezeSpectrum=true;});
 await page.getByRole('button',{name:'Play',exact:true}).click();
 await page.evaluate(()=>window.radioPreview.bridge.spectrum([1,1,1,1,1]));await expect.poll(pixels).not.toBe(idle);const active=await pixels();
 await page.evaluate(()=>window.radioPreview.bridge.deliverSpectrum({generation:window.radioPreview.ui.getState().snapshot.generation-1,levels:[0,0,0,0,0],powered:false}));
 await page.waitForTimeout(50);expect(await pixels()).toBe(active);
 await page.evaluate(()=>window.radioPreview.bridge.deliverSpectrum({generation:window.radioPreview.ui.getState().snapshot.generation,levels:[1,1,1,1,1],powered:false}));await expect.poll(pixels).toBe(idle);
 await page.evaluate(()=>window.radioPreview.bridge.spectrum([1,1,1,1,1]));await expect.poll(pixels).not.toBe(idle);
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'reconnecting'}));await expect.poll(pixels).toBe(idle);
 await page.evaluate(()=>window.radioPreview.bridge.spectrum([1,1,1,1,1]));await page.waitForTimeout(50);expect(await pixels()).toBe(idle);
 await page.getByRole('button',{name:'Stop',exact:true}).click();await expect.poll(pixels).toBe(idle);
});


test('SpectrumAnalyzer public API compatibility and pending-frame lifecycle on rc.4',async({page},testInfo)=>{
 const errors=[];page.on('pageerror',error=>errors.push(error.message));
 const result=await page.evaluate(async()=>{
  // Test-owned fixture and browser scheduling probes; no component internals are accessed.
  window.radioPreview.ui.destroy();window.radioPreview.bridge.destroy();
  const { SpectrumAnalyzer }=await import('/extensions/SpectrumAnalyzer.js');
  const { Panel,BaseControl }=await import('/vendor/taet-ui.js');
  const fixture=document.createElement('section');fixture.setAttribute('aria-label','Spectrum compatibility fixture');fixture.style.cssText='position:fixed;left:8px;top:50px;width:200px;height:130px';document.body.append(fixture);
  const nativeRAF=window.requestAnimationFrame,nativeCancel=window.cancelAnimationFrame;
  const pending=new Set();let destroyed=false,callbacksAfterDestroy=0,resizeCallbacks=0;
  window.requestAnimationFrame=callback=>{let id;id=nativeRAF.call(window,time=>{pending.delete(id);if(destroyed)callbacksAfterDestroy++;callback(time);});pending.add(id);return id;};
  window.cancelAnimationFrame=id=>{pending.delete(id);nativeCancel.call(window,id);};
  const settle=async(ms=45)=>{await new Promise(resolve=>nativeRAF.call(window,()=>nativeRAF.call(window,resolve)));await new Promise(resolve=>setTimeout(resolve,ms));};
  const owner=new Panel(fixture,{name:'spectrumCompatibilityOwner',padding:0});
  const spectrum=new SpectrumAnalyzer({name:'lifecycleSpectrum',bands:5,width:90,height:36,maxFps:60,background:'#101820',idleColor:'#203030',baselineColor:'#408060',activeColors:['#50a070','#a0c050','#d08050'],peakColor:'#f0e0a0'});
  owner.add(spectrum);spectrum.events.on('resize',()=>resizeCallbacks++);
  try{
   await settle();const canvas=spectrum.element.querySelector('canvas');const idle=canvas.toDataURL();
   const typed=new Float32Array([.25,.5,.75,1,0]);spectrum.setLevels(typed);typed.fill(0);const copied=spectrum.getLevels();const detached=spectrum.getLevels();detached[0]=.99;const detachedCopy=spectrum.getLevels();
   let invalidLengthRejected=false;try{spectrum.setLevels([1,2,3,4]);}catch{invalidLengthRejected=true;}
   const afterInvalid=spectrum.getLevels();spectrum.setLevels([-1,.4,2,NaN,Infinity]);const clamped=spectrum.getLevels();
   spectrum.setLevels([1,1,1,1,1]);await settle();const active=canvas.toDataURL();
   spectrum.setPaused(true);spectrum.clear();await settle();const pause={paused:spectrum.paused,levels:spectrum.getLevels(),pixels:canvas.toDataURL()};
   spectrum.setPaused(false);await settle();const resumed=canvas.toDataURL();
   spectrum.setLevels([1,1,1,1,1]);await settle();spectrum.hide();spectrum.clear();await settle();const hidden={visible:spectrum.isVisible(),levels:spectrum.getLevels(),pixels:canvas.toDataURL()};
   spectrum.show();await settle();const shown=canvas.toDataURL();
   spectrum.setSize({width:120,height:48});await settle();const resized={size:spectrum.getSize(),rect:{width:spectrum.element.getBoundingClientRect().width,height:spectrum.element.getBoundingClientRect().height},backing:{width:canvas.width,height:canvas.height},dpr:devicePixelRatio};
   // Leave an actual scheduled paint pending, then destroy its owning Panel.
   spectrum.setLevels([1,.8,.6,.4,.2]);const pendingBeforeDestroy=pending.size;const callbacksBeforeDestroy=resizeCallbacks;
   owner.destroy();destroyed=true;
   fixture.style.width='230px';window.dispatchEvent(new Event('resize'));document.dispatchEvent(new Event('visibilitychange'));
   await settle(120);
   return{sameIdentity:spectrum instanceof BaseControl,copied,detachedCopy,invalidLengthRejected,afterInvalid,clamped,idle,active,pause,resumed,hidden,shown,resized,pendingBeforeDestroy,pendingAfterDestroy:pending.size,callbacksBeforeDestroy,callbacksAfterDestroy,resizeCallbacks,destroyed:spectrum.destroyed,connected:canvas.isConnected,backingAfterDestroy:{width:canvas.width,height:canvas.height}};
  }finally{owner.destroy();fixture.remove();window.requestAnimationFrame=nativeRAF;window.cancelAnimationFrame=nativeCancel;}
 });
 expect(result.sameIdentity).toBe(true);expect(result.copied).toEqual([.25,.5,.75,1,0]);expect(result.detachedCopy).toEqual(result.copied);
 expect(result.invalidLengthRejected).toBe(true);expect(result.afterInvalid).toEqual(result.copied);expect(result.clamped).toEqual([0,.4,1,0,0]);
 expect(result.active).not.toBe(result.idle);expect(result.pause.paused).toBe(true);expect(result.pause.levels).toEqual([0,0,0,0,0]);expect(result.pause.pixels).toBe(result.active);expect(result.resumed).toBe(result.idle);
 expect(result.hidden.visible).toBe(false);expect(result.hidden.levels).toEqual([0,0,0,0,0]);expect(result.hidden.pixels).toBe(result.active);expect(result.shown).toBe(result.idle);
 expect(result.resized.size).toEqual({width:120,height:48});expect(result.resized.rect).toEqual({width:120,height:48});expect(result.resized.backing).toEqual({width:Math.round(120*Math.min(2,result.resized.dpr)),height:Math.round(48*Math.min(2,result.resized.dpr))});
 expect(result.pendingBeforeDestroy).toBeGreaterThan(0);expect(result.pendingAfterDestroy).toBe(0);expect(result.callbacksAfterDestroy).toBe(0);expect(result.resizeCallbacks).toBe(result.callbacksBeforeDestroy);expect(result.destroyed).toBe(true);expect(result.connected).toBe(false);expect(result.backingAfterDestroy.width*result.backingAfterDestroy.height).toBeLessThanOrEqual(1);
 await page.waitForTimeout(80);expect(errors).toEqual([]);
 for(const name of ['idle','active'])await testInfo.attach(`spectrum-${name}`,{body:Buffer.from(result[name].split(',')[1],'base64'),contentType:'image/png'});
 const {idle,active,pause,resumed,hidden,shown,...metrics}=result;await testInfo.attach('spectrum-public-lifecycle',{body:JSON.stringify(metrics,null,2),contentType:'application/json'});
});
