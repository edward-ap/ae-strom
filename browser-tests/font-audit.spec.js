import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const manifest=JSON.parse(readFileSync('web/vendor/manifest.json','utf8'));
const font=manifest.files.find(f=>f.output.endsWith('.woff2'));
const inspect=page=>page.evaluate(async()=>{const {inspectTypography}=await import('/font-audit.js');return inspectTypography();});

test('bundled standard face and typography cover all panels, native option DOM and transient values',async({page},testInfo)=>{
 const responsePromise=page.waitForResponse(r=>r.url().endsWith('/fonts/NotoSans-variable.woff2'));
 await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);
 const response=await responsePromise;expect(response.status()).toBe(200);const bytes=await response.body();expect(bytes.length).toBe(font.bytes);expect(createHash('sha256').update(bytes).digest('hex')).toBe(font.sha256);
 let result=await inspect(page);expect(result.standard).toBe(true);expect(result.loaded).toBe(true);expect(result.nativeSelectCount).toBe(11);expect(result.faces).toEqual(expect.arrayContaining([expect.objectContaining({status:'loaded',style:'normal',weight:'100 900'})]));
 expect(result.resources.every(r=>r.sameOrigin&&r.path.endsWith('/vendor/fonts/NotoSans-variable.woff2'))).toBe(true);expect(result.placeholder.family).toContain('Noto Sans');expect(result.placeholder.size).toBe('12px');
 await page.getByRole('button',{name:'Station settings',exact:true}).click();result=await inspect(page);expect(result.standard).toBe(true);expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:258});
 await page.getByRole('button',{name:'Equalizer',exact:true}).click();await page.locator('.eq-sliders input').first().focus();result=await inspect(page);expect(result.standard).toBe(true);expect((await page.locator('.radio-owner').boundingBox())).toMatchObject({width:725,height:257});
 expect(result.groups.find(g=>g.selector==='.radio-status').sizes).toEqual(['12px']);expect(result.nativePopupTypefaceMeasured).toBe(false);expect(result.nativeTitleTooltipTypefaceMeasured).toBe(false);
 await testInfo.attach('computed-fonts.json',{body:JSON.stringify(result,null,2),contentType:'application/json'});
});

test('Chromium actually renders supported scripts with the custom Noto face and records fallback separately',async({page},testInfo)=>{
 await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);
 const session=await page.context().newCDPSession(page);await session.send('DOM.enable');await session.send('CSS.enable');
 const fonts=async selector=>{const {root}=await session.send('DOM.getDocument');const {nodeId}=await session.send('DOM.querySelector',{nodeId:root.nodeId,selector});return(await session.send('CSS.getPlatformFontsForNode',{nodeId})).fonts;};
 const idle=await fonts('.radio-ticker .dui-scrolling-label__text');expect(idle.some(f=>f.isCustomFont&&f.familyName.includes('Noto Sans')&&f.glyphCount>0)).toBe(true);
 await page.evaluate(()=>window.radioPreview.bridge.update({status:'playing',requestedPlaying:true,fullTitle:'Æ Ø æ ø — Живое радио — Ελληνικά',displayTitle:'Æ Ø æ ø — Живое радио — Ελληνικά'}));
 const supported=await fonts('.radio-ticker .dui-scrolling-label__text');expect(supported.length).toBeGreaterThan(0);expect(supported.every(f=>f.isCustomFont&&f.familyName.includes('Noto Sans'))).toBe(true);
 await page.getByRole('button',{name:'Station settings',exact:true}).click();const header=await fonts('.station-header .column-label');expect(header.some(f=>f.isCustomFont&&f.familyName.includes('Noto Sans'))).toBe(true);
 const url=await fonts('[data-station-slot="0"]');const select=await fonts('.station-row select');
 await page.evaluate(()=>window.radioPreview.bridge.update({displayTitle:'Fallback probe: 漢字 🎵',fullTitle:'Fallback probe: 漢字 🎵'}));const otherScripts=await fonts('.radio-ticker .dui-scrolling-label__text');
 const actual={idle,supported,header,url,select,otherScripts,platform:await page.evaluate(()=>navigator.platform),nativePopupNotMeasured:true,nativeWKWebViewExecuted:false};writeFileSync(testInfo.outputPath('actual-fonts.json'),JSON.stringify(actual,null,2));
 await testInfo.attach('actual-fonts.json',{body:JSON.stringify(actual,null,2),contentType:'application/json'});
 await page.screenshot({path:testInfo.outputPath('coverage-fonts.png')});await session.detach();
});


test('ticker measures the completed delayed font and tears down during later font notifications',async({page})=>{
 const errors=[];page.on('pageerror',error=>errors.push(error.message));
 let releaseFont;const held=new Promise(resolve=>{releaseFont=resolve;});
 await page.route('**/fonts/NotoSans-variable.woff2',async route=>{await held;await route.continue();});
 await page.goto('/',{waitUntil:'domcontentloaded'});
 await page.waitForFunction(()=>window.radioPreview?.ui);
 const title='Delayed font ticker: WWW iii 0123456789 '.repeat(12);
 await page.evaluate(title=>window.radioPreview.bridge.update({status:'playing',requestedPlaying:true,fullTitle:title,displayTitle:title}),title);
 releaseFont();await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);
 await page.evaluate(()=>document.fonts.ready);
 await expect.poll(()=>page.evaluate(()=>{
  const track=document.querySelector('.radio-ticker .dui-scrolling-label__track');
  const text=track.querySelector('.dui-scrolling-label__text');
  const animation=track.getAnimations()[0];if(!animation)return false;
  const distance=parseFloat(getComputedStyle(text).width)+parseFloat(getComputedStyle(track).gap);
  return Math.abs(animation.effect.getTiming().duration-distance/28*1000)<1;
 })).toBe(true);
 await page.evaluate(()=>{window.radioPreview.ui.destroy();for(const type of ['loading','loadingdone','loadingerror'])document.fonts.dispatchEvent(new Event(type));});
 await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
 expect(errors).toEqual([]);
});
