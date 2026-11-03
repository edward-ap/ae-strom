import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
mkdirSync('evidence',{recursive:true});
const server=spawn(process.execPath,['scripts/preview.js','5187'],{stdio:['ignore','pipe','inherit'],windowsHide:true});
await new Promise((resolve,reject)=>{server.stdout.once('data',resolve);server.once('error',reject);});
let browser;
try{
 browser=await chromium.launch();
 for(const dpr of [1,1.5]){
 const page=await browser.newPage({viewport:{width:725,height:420},deviceScaleFactor:dpr});await page.goto('http://127.0.0.1:5187');await page.waitForFunction(()=>window.__RADIO_UI_READY__===true);
 if(dpr===1){
 await page.screenshot({path:'evidence/ui-normal-100.png',clip:{x:0,y:0,width:725,height:36}});
 await page.getByRole('button',{name:'Station settings',exact:true}).click();let box=await page.locator('.radio-owner').boundingBox();await page.screenshot({path:'evidence/ui-settings-100.png',clip:box});
 await page.getByRole('button',{name:'Equalizer',exact:true}).click();box=await page.locator('.radio-owner').boundingBox();await page.screenshot({path:'evidence/ui-equalizer-100.png',clip:box});
 await page.getByRole('button',{name:'Play',exact:true}).click();await page.waitForTimeout(100);await page.getByRole('button',{name:'Collapse player',exact:true}).click();box=await page.locator('.radio-owner').boundingBox();await page.screenshot({path:'evidence/ui-compact-100.png',clip:box});
 await page.getByRole('button',{name:'Expand player',exact:true}).click();await page.getByRole('button',{name:'Station settings',exact:true}).click();
 }else{await page.getByRole('button',{name:'Play',exact:true}).click();await page.getByRole('button',{name:'Station settings',exact:true}).click();}
 const editor=page.locator('[data-station-slot="5"]');await editor.fill('https://synthetic.invalid/'+'segment0123456789/'.repeat(42)+'?end=synthetic');await editor.press('Home');const box=await editor.boundingBox();await page.mouse.move(box.x+8,box.y+box.height/2);await page.mouse.down();await page.mouse.move(box.x+box.width+30,box.y+box.height/2,{steps:18});await page.waitForTimeout(180);await page.screenshot({path:`evidence/ui-selection-${dpr===1?'100':'150'}.png`});await page.mouse.up();await page.close();
 }
}finally{await browser?.close();server.kill();}
console.log('Saved normal/settings/EQ/compact and active long-URL selection evidence.');
