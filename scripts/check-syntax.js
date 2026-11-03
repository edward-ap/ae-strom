import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
const files=[];
function walk(dir){for(const item of readdirSync(dir,{withFileTypes:true})){if(item.name==='vendor')continue;const path=join(dir,item.name);if(item.isDirectory())walk(path);else if(path.endsWith('.js'))files.push(path);}}
for(const dir of ['web','scripts','browser-tests'])walk(dir);
for(const file of files){const result=spawnSync(process.execPath,['--check',file],{encoding:'utf8'});if(result.status!==0){console.error(result.stderr);process.exit(1);}}
console.log(`Syntax checked ${files.length} application, extension and harness files.`);
