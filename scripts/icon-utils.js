import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { inflateSync, deflateSync } from 'node:zlib';
export const sha256 = data => createHash('sha256').update(data).digest('hex');
const signature=Buffer.from('89504e470d0a1a0a','hex');
const assert=(condition,message)=>{if(!condition)throw Error(message);};
export function png(data){
 assert(data.subarray(0,8).equals(signature),'Expected PNG');
 let offset=8,width,height,parts=[];
 while(offset<data.length){const length=data.readUInt32BE(offset),type=data.toString('ascii',offset+4,offset+8),body=data.subarray(offset+8,offset+8+length);assert(offset+length+12<=data.length,'Truncated PNG');
  if(type==='IHDR'){width=body.readUInt32BE(0);height=body.readUInt32BE(4);assert(body[8]===8&&body[9]===6&&body[12]===0,'Expected noninterlaced RGBA8 PNG');}
  if(type==='IDAT')parts.push(body);offset+=length+12;
 }
 const stride=width*4,raw=inflateSync(Buffer.concat(parts));assert(raw.length===(stride+1)*height,'Unexpected PNG scanline size');
 const rgba=Buffer.alloc(stride*height);let at=0;
 const paeth=(a,b,c)=>{const p=a+b-c,pa=Math.abs(p-a),pb=Math.abs(p-b),pc=Math.abs(p-c);return pa<=pb&&pa<=pc?a:pb<=pc?b:c;};
 for(let y=0;y<height;y++){const filter=raw[at++];assert(filter<=4,'Unknown PNG filter');for(let x=0;x<stride;x++){
  const i=y*stride+x,left=x>=4?rgba[i-4]:0,up=y?rgba[i-stride]:0,upperLeft=y&&x>=4?rgba[i-stride-4]:0;
  const prediction=filter===0?0:filter===1?left:filter===2?up:filter===3?Math.floor((left+up)/2):paeth(left,up,upperLeft);rgba[i]=(raw[at++]+prediction)&255;
 }}
 return {width,height,rgba};
}
export function stats(image){
 const {width,height,rgba}=image;const corners=[3,(width-1)*4+3,((height-1)*width)*4+3,(width*height-1)*4+3].map(i=>rgba[i]);
 let transparent=0,partial=0,solid=0;for(let i=3;i<rgba.length;i+=4){if(rgba[i]===0)transparent++;else if(rgba[i]===255)solid++;else partial++;}
 const centerAlpha=rgba[(Math.floor(height/2)*width+Math.floor(width/2))*4+3];
 assert(corners.every(a=>a===0),'Opaque exterior corner');assert(transparent>0&&centerAlpha>=240,'Missing transparency or tile interior');
 return {width,height,corners,centerAlpha,transparent,partial,solid,rgbaSha256:sha256(rgba)};
}
export function ico(data){
 assert(data.readUInt16LE(0)===0&&data.readUInt16LE(2)===1,'Invalid ICO');const count=data.readUInt16LE(4),frames=[];
 for(let i=0;i<count;i++){const e=6+16*i,size=data.readUInt32LE(e+8),offset=data.readUInt32LE(e+12);const frame=png(data.subarray(offset,offset+size));assert(frame.width===(data[e]||256)&&frame.height===(data[e+1]||256),'ICO dimensions mismatch');frames.push(stats(frame));}
 assert([16,24,32,48,64,256].every(n=>frames.some(f=>f.width===n&&f.height===n)),'Missing Windows icon size');return frames;
}
export function icns(data){
 assert(data.toString('ascii',0,4)==='icns'&&data.readUInt32BE(4)===data.length,'Invalid ICNS');let offset=8;const members=[];
 while(offset<data.length){const type=data.toString('ascii',offset,offset+4),length=data.readUInt32BE(offset+4),body=data.subarray(offset+8,offset+length);assert(length>=8&&offset+length<=data.length,'Invalid ICNS member');
  if(body.subarray(0,8).equals(signature))members.push({type,...stats(png(body))});
  else if(type==='s8mk'||type==='l8mk'){const size=type==='s8mk'?16:32;assert(body.length===size*size,'Invalid legacy alpha mask');const corners=[body[0],body[size-1],body[(size-1)*size],body[body.length-1]];assert(corners.every(a=>a===0),'Opaque legacy ICNS corner');members.push({type,width:size,height:size,corners});}
  else assert(type==='is32'||type==='il32','Unknown ICNS encoding');offset+=length;
 }
 assert(members.some(m=>m.width===1024)&&members.some(m=>m.type==='s8mk')&&members.some(m=>m.type==='l8mk'),'Incomplete desktop ICNS');return members;
}
export function fnv64(bytes){let value=0xcbf29ce484222325n;for(const b of bytes)value=BigInt.asUintN(64,(value^BigInt(b))*0x100000001b3n);return value.toString(16).padStart(16,'0');}
export function inspect(path){const bytes=readFileSync(path),kind=path.endsWith('.ico')?'ico':path.endsWith('.icns')?'icns':'png';return {bytes:bytes.length,sha256:sha256(bytes),kind,images:kind==='png'?[stats(png(bytes))]:kind==='ico'?ico(bytes):icns(bytes)};}

// Resampling can round effectively transparent pixels to 1/255. This is
// format-stage quantization cleanup only: RGB and all alpha >=2 stay unchanged.
const crcTable=Array.from({length:256},(_,n)=>{let c=n;for(let k=0;k<8;k++)c=c&1?0xedb88320^(c>>>1):c>>>1;return c>>>0;});
function chunk(type,body){const label=Buffer.from(type),out=Buffer.alloc(body.length+12);out.writeUInt32BE(body.length);label.copy(out,4);body.copy(out,8);let crc=0xffffffff;for(const b of Buffer.concat([label,body]))crc=crcTable[(crc^b)&255]^(crc>>>8);out.writeUInt32BE((crc^0xffffffff)>>>0,out.length-4);return out;}
function encodePng({width,height,rgba}){const header=Buffer.alloc(13);header.writeUInt32BE(width);header.writeUInt32BE(height,4);header[8]=8;header[9]=6;const stride=width*4,raw=Buffer.alloc((stride+1)*height);for(let y=0;y<height;y++)rgba.copy(raw,y*(stride+1)+1,y*stride,(y+1)*stride);return Buffer.concat([signature,chunk('IHDR',header),chunk('IDAT',deflateSync(raw,{level:9})),chunk('IEND',Buffer.alloc(0))]);}
function cleanPng(data){const image=png(data);let changed=false;for(let i=3;i<image.rgba.length;i+=4)if(image.rgba[i]===1){image.rgba[i]=0;changed=true;}return changed?encodePng(image):data;}
export function cleanGenerated(data,extension){
 if(extension==='.png')return cleanPng(data);
 if(extension==='.ico'){
  const count=data.readUInt16LE(4),header=Buffer.from(data.subarray(0,6+16*count)),parts=[];let offset=header.length;
  for(let i=0;i<count;i++){const entry=6+i*16,size=data.readUInt32LE(entry+8),start=data.readUInt32LE(entry+12),payload=cleanPng(data.subarray(start,start+size));header.writeUInt32LE(payload.length,entry+8);header.writeUInt32LE(offset,entry+12);offset+=payload.length;parts.push(payload);}return Buffer.concat([header,...parts]);
 }
 if(extension==='.icns'){
  let offset=8;const parts=[];
  while(offset<data.length){const type=data.toString('ascii',offset,offset+4),length=data.readUInt32BE(offset+4);let body=data.subarray(offset+8,offset+length);
   if(body.subarray(0,8).equals(signature))body=cleanPng(body);
   else if(type==='s8mk'||type==='l8mk')body=Buffer.from(body.map(a=>a<=1?0:a));
   parts.push({type,body});offset+=length;
  }
  parts.sort((a,b)=>a.type<b.type?-1:a.type>b.type?1:0);
  const chunks=parts.map(({type,body})=>{const h=Buffer.alloc(8);h.write(type);h.writeUInt32BE(body.length+8,4);return Buffer.concat([h,body]);});
  const head=Buffer.alloc(8);head.write('icns');head.writeUInt32BE(8+chunks.reduce((n,p)=>n+p.length,0),4);return Buffer.concat([head,...chunks]);
 }
 throw Error('Unsupported icon format');
}
