import { test, expect } from '@playwright/test';
test.beforeEach(async ({page}) => { await page.goto('/'); await page.waitForFunction(() => window.__RADIO_UI_READY__); });
const openEQ = page => page.getByRole('button', {name:'Equalizer',exact:true}).click();
const gain = (page,index) => page.locator(`[data-dui-control-id="eqBand${index}"] input`);

test('wider Settings presets borrow URL width and preserve square arrow and drafts', async ({page}) => {
 await page.getByRole('button',{name:'Station settings',exact:true}).click();
 const widths=await page.locator('.station-row select').evaluateAll(es=>es.map(e=>{const s=getComputedStyle(e),a=getComputedStyle(e.parentElement,'::after'),b=e.getBoundingClientRect();return{width:b.width,height:b.height,square:[a.width,a.height],content:b.width-parseFloat(s.paddingLeft)-parseFloat(s.paddingRight)-parseFloat(s.borderLeftWidth)-parseFloat(s.borderRightWidth),font:s.fontSize};}));
 for(const w of widths)expect(w).toEqual({width:110,height:24,square:['24px','24px'],content:78,font:'12px'});
 const fits=await page.evaluate(()=>{const s=getComputedStyle(document.querySelector('.station-row select')),c=document.createElement('canvas').getContext('2d');c.font=s.font;return ['Acoustic','Electronic','Full Treble','Soft Rock'].map(name=>({name,width:c.measureText(name).width}));});for(const item of fits)expect(item.width).toBeLessThanOrEqual(78);
 const url=page.locator('[data-station-slot="5"]');await url.fill('https://synthetic.invalid/unsaved?q=%D0%96');await page.getByRole('button',{name:'Browse bank Z',exact:true}).click();await page.getByRole('button',{name:'Browse bank A',exact:true}).click();await expect(url).toHaveValue('https://synthetic.invalid/unsaved?q=%D0%96');
 expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:258});expect(await page.evaluate(()=>window.radioPreview.bridge.actions.some(a=>['setPlaying','selectStation'].includes(a.type)))).toBe(false);
});

test('EQ common scale, bottom labels, grips and reduced height have no permanent value row', async ({page}) => {
 await openEQ(page);await expect(page.locator('.eq-panel .panel-hint')).toHaveCount(0);await expect(page.locator('.eq-sliders .dui-trackbar__tick')).toHaveCount(0);
 const geometry=await page.evaluate(()=>{
  const controls=[...document.querySelectorAll('.eq-sliders .dui-trackbar')];return{controls:controls.map(e=>{const i=e.querySelector('input'),b=e.getBoundingClientRect(),r=i.getBoundingClientRect(),l=e.querySelector('.dui-trackbar__label');return{rect:b.toJSON(),input:r.toJSON(),label:l.textContent,labelTop:l.getBoundingClientRect().top,min:i.min,max:i.max,step:i.step,shape:e.dataset.thumbShape,thumb:getComputedStyle(e).getPropertyValue('--dui-thumb-background'),valueVisible:getComputedStyle(e.querySelector('.dui-trackbar__value')).visibility};}),lines:[...document.querySelectorAll('.eq-grid-line')].map(e=>({db:Number(e.dataset.eqDb),height:e.getBoundingClientRect().height,y:e.getBoundingClientRect().y+e.getBoundingClientRect().height/2,major:e.classList.contains('eq-grid-line--major')}))};
 });
 expect(geometry.controls.map(c=>c.label)).toEqual(['Preamp','32','64','125','250','500','1K','2K','4K','8K','16KHz']);
 for(const c of geometry.controls){expect(c.min).toBe('-15');expect(c.max).toBe('15');expect(c.step).toBe('1');expect(c.shape).toBe('grip');expect(c.thumb).toContain('#33363c');expect(c.valueVisible).toBe('hidden');expect(c.labelTop-c.input.bottom).toBe(2);}
 const gaps=geometry.controls.slice(1).map((c,i)=>c.rect.x-geometry.controls[i].rect.right);expect(gaps[0]/gaps[1]).toBeGreaterThan(1.8);expect(gaps[0]/gaps[1]).toBeLessThan(2.2);
 expect(geometry.lines.map(l=>l.db)).toEqual([15,10,5,0,-5,-10,-15]);for(const line of geometry.lines){expect(line.height).toBe(line.major?2:1);const r=geometry.controls[0].input;expect(Math.abs(line.y-(r.y+12+(15-line.db)/30*(r.height-24)))).toBeLessThan(.1);}
 expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:257});
 await expect(page.locator('.radio-volume')).toHaveAttribute('data-thumb-shape','grip');expect(await page.locator('.radio-volume').evaluate(e=>getComputedStyle(e).getPropertyValue('--dui-thumb-background'))).toContain('#33363c');
});

test('shared grid follows actual painted native grip centers at all seven values', async ({page}) => {
 await openEQ(page);
 for(const db of [15,10,5,0,-5,-10,-15]){
  await page.evaluate(db=>{const s=window.radioPreview.ui.getState().snapshot;window.radioPreview.bridge.update({eq:{...s.eq,preamp:db,bands:Array(10).fill(db)}});},db);
  for(const index of [0,10]){
   const field=gain(page,index),rect=await field.boundingBox(),png=await field.screenshot();
   const pixels=await page.evaluate(async png=>{const b=await createImageBitmap(new Blob([Uint8Array.from(atob(png),c=>c.charCodeAt(0))],{type:'image/png'}));const c=document.createElement('canvas');c.width=b.width;c.height=b.height;const ctx=c.getContext('2d');ctx.drawImage(b,0,0);const a=ctx.getImageData(0,0,c.width,c.height).data,ys=[];let gray=0;for(let y=0;y<c.height;y++){let dark=0;for(let x=Math.ceil(c.width/2-5*devicePixelRatio);x<c.width/2+5*devicePixelRatio;x++){const i=(y*c.width+x)*4;if(Math.abs(a[i]-51)<3&&Math.abs(a[i+1]-54)<3&&Math.abs(a[i+2]-60)<3)dark++;if(a[i]===136&&a[i+1]===136&&a[i+2]===136)gray++;}if(dark>=6*devicePixelRatio)ys.push(y);}return{center:(Math.min(...ys)+Math.max(...ys)+1)/2/devicePixelRatio,gray,rows:ys.length};},png.toString('base64'));
   expect(pixels.rows).toBeGreaterThan(8);expect(pixels.gray).toBeGreaterThan(20);
   const line=await page.locator(`.eq-grid-line[data-eq-db="${db}"]`).boundingBox();expect(Math.abs(rect.y+pixels.center-line.y-line.height/2),JSON.stringify({db,index,pixels,rect,line})).toBeLessThanOrEqual(1.1);
  }
 }
 expect(await page.evaluate(()=>window.radioPreview.bridge.actions)).toEqual([]);
});

test('hover, keyboard and drag values appear above without layout jumps or playback', async ({page}) => {
 await openEQ(page);const first=page.locator('[data-dui-control-id="eqBand0"]'),value=first.locator('.dui-trackbar__value');await first.hover();await expect(value).toHaveCSS('visibility','visible');await expect(value).toHaveText('0 dB');expect((await value.boundingBox()).y+(await value.boundingBox()).height).toBeLessThanOrEqual((await gain(page,0).boundingBox()).y);
 await page.mouse.move(850,500);await expect(value).toHaveCSS('visibility','hidden');const input=gain(page,1);await input.focus();await input.press('Home');await expect(input).toHaveValue('-15');await input.press('ArrowUp');await expect(input).toHaveValue('-14');await input.press('End');await expect(input).toHaveValue('15');await input.press('ArrowDown');await expect(input).toHaveValue('14');await expect(page.locator('[data-dui-control-id="eqBand1"] .dui-trackbar__value')).toHaveText('+14 dB');await expect(input).toHaveCSS('outline-style','solid');
 const moving=gain(page,2),box=await moving.boundingBox();await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();await expect(page.locator('[data-dui-control-id="eqBand2"]')).toHaveAttribute('data-dragging','true');await page.mouse.move(box.x+box.width/2,box.y+30,{steps:8});await page.mouse.up();expect(Number(await moving.inputValue())).toBeGreaterThanOrEqual(8);expect(Number(await moving.inputValue())).toBeLessThanOrEqual(12);await expect(page.locator('[data-dui-control-id="eqBand2"]')).not.toHaveAttribute('data-dragging','true');
 expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:257});expect(await page.evaluate(()=>window.radioPreview.bridge.actions.every(a=>a.type==='setEq'))).toBe(true);expect(await page.evaluate(()=>window.radioPreview.bridge.dragCount)).toBe(0);
 const retained=await moving.inputValue();await page.getByRole('button',{name:'Collapse player',exact:true}).click();expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:102,height:36});await page.getByRole('button',{name:'Expand player',exact:true}).click();await expect(moving).toHaveValue(retained);
});
