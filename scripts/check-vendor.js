import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const read=p=>JSON.parse(readFileSync(p,'utf8'));
const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
const m=read('web/vendor/manifest.json'),pin=read('provenance/taet-ui-rc7-source.json');
if(m.package!=='taet-ui'||m.version!=='0.6.0-rc.7'||m.profile!=='components'||m.sourceCommit!=='53c5b177a5b68f1c1779afe97316480c65101840'||m.integrity!==pin.integrity||m.tarballSha256!==pin.tarballSha256)throw Error('Unpinned framework delivery');
if(m.files.length!==6)throw Error('Unexpected runtime asset count');
for(const f of m.files){const p='web/vendor/'+f.output;if(hash(p)!==f.sha256||readFileSync(p).length!==f.bytes)throw Error('Vendor asset mismatch: '+f.output);}
for(const f of m.typeFiles)if(hash('web/vendor/'+f.path)!==f.sha256)throw Error('Vendor declaration mismatch: '+f.path);
if(hash('web/vendor/taet-ui.d.ts')!==m.facadeSha256)throw Error('Vendor facade mismatch');
const pkg=read('package.json'),locked=read('package-lock.json').packages['node_modules/taet-ui'];
if(Object.keys(pkg.dependencies||{}).length||pkg.devDependencies['taet-ui']!==m.version||locked?.version!==m.version||locked.integrity!==m.integrity)throw Error('Runtime dependency or package lock mismatch');
console.log(`Verified taet-ui ${m.version}:6 runtime assets,${m.typeFiles.length} declarations,facade and npm integrity; zero UI runtime dependencies.`);
