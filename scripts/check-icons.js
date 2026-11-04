import {readFileSync} from 'node:fs';
import {resolve,dirname,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {sha256,inspect,png,fnv64} from './icon-utils.js';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),read=p=>JSON.parse(readFileSync(join(root,p),'utf8'));
const manifest=read('icons/manifest.json'),provenance=read(manifest.sourceProvenance),pkg=read('package.json'),lock=read('package-lock.json');
if(manifest.generator.version!=='2.11.4'||pkg.devDependencies['@tauri-apps/cli']!=='2.11.4'||lock.packages['node_modules/@tauri-apps/cli'].version!=='2.11.4')throw Error('Unpinned icon generator');
for(const source of [provenance.original,provenance.transparentMaster])if(sha256(readFileSync(join(root,'icons',source.path)))!==source.sha256)throw Error('Icon source modified without reviewed provenance');
inspect(join(root,'icons/source.png'));let images=0;
for(const entry of manifest.files){const actual=inspect(join(root,entry.path));if(JSON.stringify(actual)!==JSON.stringify(Object.fromEntries(Object.entries(entry).filter(([k])=>k!=='path'))))throw Error('Icon manifest mismatch: '+entry.path);images+=actual.images.length;}
const window=png(readFileSync(join(root,manifest.windowIcon.path)));
if(window.width!==32||window.height!==32||fnv64(window.rgba)!==manifest.windowIcon.rgbaFnv1a64)throw Error('Window PNG identity mismatch');
const config=read('tauri.conf.json');const expected=['icons/32x32.png','icons/128x128.png','icons/128x128@2x.png','icons/icon.png','icons/icon.icns','icons/icon.ico'];
if(JSON.stringify(config.bundle.icon)!==JSON.stringify(expected))throw Error('Tauri icon target mismatch');
if(sha256(readFileSync(join(root,'web/app-icon.png')))!==sha256(readFileSync(join(root,'icons/32x32.png'))))throw Error('Stale web favicon');
if(!readFileSync(join(root,'web/index.html'),'utf8').includes('href="app-icon.png"'))throw Error('Web favicon not linked');
console.log(`Verified original/master, ${manifest.files.length} generated assets and ${images} PNG/ICO/ICNS alpha representations, Tauri window PNG identity and favicon.`);
