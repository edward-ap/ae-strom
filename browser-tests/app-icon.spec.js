import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
const manifest=JSON.parse(readFileSync(new URL('../icons/manifest.json',import.meta.url),'utf8'));
test('application favicon loads the new transparent radio pixels',async({page})=>{
 await page.goto('/');
 await expect(page.locator('link[rel="icon"]')).toHaveAttribute('href','app-icon.png');
 const response=await page.request.get('/app-icon.png');expect(response.headers()['content-type']).toBe('image/png');
 const actual=await page.evaluate(async()=>{
  const image=new Image();image.src='app-icon.png';await image.decode();
  const canvas=document.createElement('canvas');canvas.width=image.width;canvas.height=image.height;const context=canvas.getContext('2d');context.drawImage(image,0,0);
  const rgba=context.getImageData(0,0,image.width,image.height).data;
  // Browser canvas can round semi-transparent RGB through premultiplication.
  return {width:image.width,height:image.height,corners:[rgba[3],rgba[(image.width-1)*4+3],rgba[((image.height-1)*image.width)*4+3],rgba[rgba.length-1]],centerAlpha:rgba[(16*image.width+16)*4+3]};
 });
 expect(actual.width).toBe(manifest.windowIcon.width);expect(actual.height).toBe(manifest.windowIcon.height);expect(actual.corners).toEqual([0,0,0,0]);expect(actual.centerAlpha).toBeGreaterThanOrEqual(240);
});
