import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve,dirname,join,extname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {sha256,inspect,png,fnv64,cleanGenerated} from './icon-utils.js';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..'),icons=join(root,'icons');
const provenance=JSON.parse(readFileSync(join(icons,'source-provenance.json'),'utf8'));
for(const source of [provenance.original,provenance.transparentMaster])if(sha256(readFileSync(join(icons,source.path)))!==source.sha256)throw Error('Icon source/provenance mismatch; update the reviewed original/master pair first');
inspect(join(icons,'source.png'));
const cliRoot=join(root,'node_modules/@tauri-apps/cli'),pkg=JSON.parse(readFileSync(join(cliRoot,'package.json'),'utf8'));
if(pkg.version!=='2.11.4')throw Error('Pinned Tauri icon CLI 2.11.4 required');
const stage=join(root,'.gen/app-icon-stage'),small=join(root,'.gen/app-icon-small');mkdirSync(stage,{recursive:true});mkdirSync(small,{recursive:true});
function run(args){const result=spawnSync(process.execPath,[join(cliRoot,'tauri.js'),'icon',join(icons,'source.png'),...args],{cwd:root,stdio:'inherit'});if(result.status!==0)throw Error('Tauri icon generation failed');}
run(['--output',stage]);run(['--output',small,...[16,24,48,64].flatMap(size=>['--png',String(size)])]);
const files=[];
function put(path,data){if(!existsSync(path)||!readFileSync(path).equals(data))writeFileSync(path,data);}
function publish(from,to){put(to,cleanGenerated(readFileSync(from),extname(to)));inspect(to);}
for(const name of ['32x32.png','128x128.png','128x128@2x.png','icon.png','icon.ico','icon.icns']){publish(join(stage,name),join(icons,name));files.push('icons/'+name);}
for(const name of ['16x16.png','24x24.png','48x48.png','64x64.png']){publish(join(small,name),join(icons,name));files.push('icons/'+name);}
put(join(root,'web/app-icon.png'),readFileSync(join(icons,'32x32.png')));files.push('web/app-icon.png');
const window=png(readFileSync(join(icons,'32x32.png')));
const manifest={schemaVersion:1,generator:{tool:'@tauri-apps/cli',version:pkg.version,alphaQuantizationCleanup:'0..1 to 0 after resize; RGB and alpha 2..255 unchanged',icnsOrder:'sorted by four-byte member type'},sourceProvenance:'icons/source-provenance.json',windowIcon:{path:'icons/32x32.png',width:window.width,height:window.height,rgbaFnv1a64:fnv64(window.rgba)},files:files.sort().map(path=>({path,...inspect(join(root,path))}))};
put(join(icons,'manifest.json'),Buffer.from(JSON.stringify(manifest,null,2)+String.fromCharCode(10)));
console.log('Generated desktop PNG/ICO/ICNS + web favicon; all corners transparent; original/master preserved.');
