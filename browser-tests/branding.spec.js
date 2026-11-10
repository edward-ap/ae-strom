import {test,expect} from '@playwright/test';
test('Æ Strøm identity and one pinned Tæt component identity',async({page})=>{
 await page.goto('/');await page.waitForFunction(()=>window.__RADIO_UI_READY__);await expect(page).toHaveTitle('Æ Strøm');
 const identity=await page.evaluate(async()=>{const {BaseControl,Panel,icons}=await import('/vendor/taet-ui.js');const {ChannelIndicator}=await import('/channel-indicator.js');const {SpectrumAnalyzer}=await import('/extensions/SpectrumAnalyzer.js');const manifest=await(await fetch('/vendor/manifest.json')).json();return{channel:new ChannelIndicator() instanceof BaseControl,spectrum:new SpectrumAnalyzer() instanceof BaseControl,panel:new Panel() instanceof BaseControl,icons:typeof icons.register==='function',name:manifest.package,version:manifest.version};});
 expect(identity).toEqual({channel:true,spectrum:true,panel:true,icons:true,name:'taet-ui',version:'0.6.0-rc.7'});
 expect(await page.evaluate(()=>window.radioPreview.ui.getState().snapshot.requestedPlaying)).toBe(false);expect(await page.evaluate(()=>window.radioPreview.bridge.actions)).toEqual([]);
});
