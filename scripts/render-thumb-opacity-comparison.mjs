import sharp from 'sharp';
import { readFile, mkdir } from 'node:fs/promises';
const dir='target/thumb-opacity-comparison';
await mkdir(dir,{recursive:true});
const svg=(w,h,b)=>Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">${b}</svg>`);
const backgrounds=[
 ['Warm', '<rect width="72" height="152" fill="#aca48e"/><path d="M0 0H34V152H0Z" fill="#57584e"/><path d="M25 0H44V152H25Z" fill="#d8d1bd"/>'],
 ['Dark','<rect width="72" height="152" fill="#293b4e"/><path d="M0 10L72 60V100L0 50Z" fill="#657b85"/>'],
 ['Bright','<rect width="72" height="152" fill="#dfc799"/><path d="M0 0L72 100V152L0 52Z" fill="#c76e43"/>']
];
for(const scale of [1,3]){
 const col=72*scale+24,row=152*scale+30,w=80+col*3,h=65+row*3;
 let body='<rect width="100%" height="100%" fill="#292929"/>';
 const layers=[];
 for(const [i,opacity] of [40,60,80].entries()){
  const x=80+i*col;
  body+=`<text x="${x}" y="26" fill="white" font-family="Segoe UI" font-size="15">${opacity}%${opacity===80?' (current)':''}</text>`;
  for(const [r,[name,bg]] of backgrounds.entries()){
   const y=50+r*row;
   if(!i)body+=`<text x="8" y="${y+20}" fill="white" font-family="Segoe UI" font-size="13">${name}</text>`;
   const keys=[];
   for(const [j,half] of ['upper','lower'].entries()){
    let source=await readFile(`target/transparent-key-lit-preview/1-${half}.svg`,'utf8');
    const re=/(<circle[^>]*r="25" fill="black" fill-opacity=")0\.8(" stroke=)/;
    if(!re.test(source))throw Error('Production thumb not found');
    source=source.replace(re,`$1${opacity/100}$2`);
    keys.push({input:await sharp(Buffer.from(source)).resize(72,72).png().toBuffer(),left:0,top:j*80});
   }
   const pair=await sharp(svg(72,152,bg)).composite(keys).png().toBuffer();
   layers.push({input:await sharp(pair).resize(72*scale,152*scale,{kernel:'nearest'}).png().toBuffer(),left:x,top:y});
  }
 }
 await sharp(svg(w,h,body)).composite(layers).png().toFile(`${dir}/comparison-${scale}x.png`);
}
