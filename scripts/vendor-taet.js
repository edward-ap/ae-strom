import {build,version as esbuildVersion} from 'esbuild';
import {createRequire} from 'node:module';
import {readFileSync,writeFileSync,mkdirSync,copyFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve,dirname,join,relative} from 'node:path';
import {fileURLToPath} from 'node:url';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const read=p=>JSON.parse(readFileSync(p,'utf8'));
const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
const pin=read(join(root,'provenance/taet-ui-rc4-source.json'));
const locked=read(join(root,'package-lock.json')).packages['node_modules/taet-ui'];
if(pin.version!=='0.6.0-rc.4'||pin.package!=='taet-ui'||locked?.version!==pin.version||locked.integrity!==pin.integrity)throw Error('Published taet-ui package pin mismatch');
if(esbuildVersion!==pin.buildTool.esbuild)throw Error('Pinned esbuild required');
const packageRoot=resolve(dirname(createRequire(import.meta.url).resolve('taet-ui')),'../..');
const pkg=read(join(packageRoot,'package.json'));
if(pkg.name!==pin.package||pkg.version!==pin.version||Object.keys(pkg.dependencies||{}).length)throw Error('Unexpected framework package/runtime dependencies');
for(const f of pin.sourceFiles){const path=resolve(packageRoot,f.path);if(!path.startsWith(packageRoot+'/')&&!path.startsWith(packageRoot+'\\'))throw Error('Unsafe source manifest path');if(hash(path)!==f.sha256||readFileSync(path).length!==f.bytes)throw Error('Published source bytes differ: '+f.path);}
const destination=join(root,'web/vendor'),expected=read(join(destination,'manifest.json'));
if(expected.integrity!==pin.integrity||expected.sourceCommit!==pin.sourceCommit)throw Error('Pinned vendor manifest required before regeneration');
const stage=join(root,'.gen/vendor-stage');mkdirSync(stage,{recursive:true});
const entry=join(root,'.gen/taet-components-entry.mjs');writeFileSync(entry,"export * from 'taet-ui/foundation';\nexport * from 'taet-ui/components';\n");
const common={absWorkingDir:root,bundle:true,format:'esm',platform:'browser',target:pin.buildTool.target,sourcemap:true,sourcesContent:true,charset:'utf8',legalComments:'eof',keepNames:true,minify:true,logLevel:'warning'};
const js=await build({...common,entryPoints:[entry],outfile:join(stage,'taet-ui.min.js'),metafile:true,banner:{js:'/*! Tæt UI 0.6.0-rc.4 | MIT | see LICENSE and THIRD_PARTY_NOTICES.md */'}});
if(Object.values(js.metafile.outputs).some(o=>o.imports?.length))throw Error('Runtime external JS import introduced');
await build({...common,entryPoints:[join(packageRoot,'src/ui/taet-ui.css')],outfile:join(stage,'taet-ui.min.css'),loader:{'.woff2':'file'},assetNames:'fonts/[name]',banner:{css:'/*! Tæt UI 0.6.0-rc.4 | MIT; bundled font OFL */'}});
const mappings=[['taet-ui.min.js','taet-ui.js',stage],['taet-ui.min.css','taet-ui.css',stage],['LICENSE','LICENSE',packageRoot],['THIRD_PARTY_NOTICES.md','THIRD_PARTY_NOTICES.md',packageRoot],['fonts/NotoSans-variable.woff2','fonts/NotoSans-variable.woff2',stage],['src/ui/fonts/NotoSans-OFL.txt','fonts/NotoSans-OFL.txt',packageRoot]];
for(const [input,output,base] of mappings){const path=join(base,input),f=expected.files.find(x=>x.output===output);if(!f||hash(path)!==f.sha256||readFileSync(path).length!==f.bytes)throw Error('Canonical built artifact differs: '+output);}
for(const [input,output,base] of mappings){const target=join(destination,output);mkdirSync(dirname(target),{recursive:true});copyFileSync(join(base,input),target);}
const actualTypes=[];
function declarations(dir){for(const e of readdirSync(dir,{withFileTypes:true})){const source=join(dir,e.name);if(e.isDirectory())declarations(source);else if(e.name.endsWith('.d.ts')){const output=join(destination,'types',relative(join(packageRoot,'src'),source));mkdirSync(dirname(output),{recursive:true});copyFileSync(source,output);actualTypes.push(relative(destination,output).replaceAll('\\','/'));}}}
declarations(join(packageRoot,'src'));
for(const f of expected.typeFiles)if(!actualTypes.includes(f.path)||hash(join(destination,f.path))!==f.sha256)throw Error('Pinned declaration differs: '+f.path);
if(actualTypes.length!==expected.typeFiles.length)throw Error('Unexpected declaration set');
const facade="export * from './types/foundation/index.js';\nexport * from './types/components/index.js';\n";
if(createHash('sha256').update(facade).digest('hex')!==expected.facadeSha256)throw Error('Type facade pin mismatch');
writeFileSync(join(destination,'taet-ui.d.ts'),facade);
console.log(`Verified published ${pin.package}@${pin.version} integrity, ${pin.sourceFiles.length} source files, ${expected.files.length} runtime assets and ${actualTypes.length} declarations; components ESM regenerated.`);
