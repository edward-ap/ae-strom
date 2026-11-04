import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { resolve, extname, sep } from 'node:path';
const root=resolve('web'),port=Number(process.env.PORT||process.argv[2]||5186);
const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.json':'application/json','.woff2':'font/woff2','.png':'image/png'};
createServer(async(req,res)=>{try{let path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(path==='/')path='/index.html';const file=resolve(root,'.'+path);if(!file.startsWith(root+sep))throw Error('path');if(!(await stat(file)).isFile())throw Error('file');res.writeHead(200,{'Content-Type':mime[extname(file)]||'application/octet-stream','Cache-Control':'no-store'});res.end(await readFile(file));}catch{res.writeHead(404);res.end('Not found');}}).listen(port,'127.0.0.1',()=>console.log(`Synthetic preview http://127.0.0.1:${port} (no audio)`));
