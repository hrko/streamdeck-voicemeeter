import sharp from 'sharp';
import {readFile,writeFile} from 'node:fs/promises';
const dir='target/transparent-key-preview';
const svg=(w,h,b)=>Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">${b}</svg>`);
function proposal(s){
 const m=s.match(/<circle cx="([\d.]+)" cy="72" r="34" fill="[^"]+"\/>/);
 if(!m) throw Error('Missing thumb geometry');
 const x=m[1];
 const hole=`<defs><mask id="thumb-hole" maskUnits="userSpaceOnUse" x="-100" y="-100" width="600" height="400"><rect x="-100" y="-100" width="600" height="400" fill="white"/><circle cx="${x}" cy="72" r="25" fill="black"/></mask><mask id="rail-hole" maskUnits="userSpaceOnUse" x="-100" y="-100" width="600" height="400"><rect x="-100" y="-100" width="600" height="400" fill="white"/><path d="M36.5 72H252" stroke="black" stroke-width="16" stroke-linecap="round"/></mask></defs>`;
 s=s.replace(/(<svg[^>]*>)/,'$1'+hole);
 s=s.replace(/(<g transform="translate\(0 24\)">)/,'$1<g mask="url(#thumb-hole)">');
 s=s.replace(/(<path d="M36.5 72H252"[^>]*stroke-width="28"[^>]*)(\/>)/,'$1 mask="url(#rail-hole)"$2');
 s=s.replace(/stroke="#000000" stroke-width="16"/g,'stroke="#000000" stroke-opacity="0.4" stroke-width="16"');
 s=s.replace(m[0],'</g>');
 s=s.replace(/(<circle cx="[\d.]+" cy="72" r="25" fill=")[^"]+(" stroke=")[^"]+(" stroke-width="6"\/>)/,'$1#000000" fill-opacity="0.8" stroke="#ffffff" stroke-width="6"/>');
 s=s.replace(/(height="24" fill="#000000")/g,'$1 fill-opacity="0.4"');
 s=s.replaceAll('fill="#1b1b1b"', 'fill="#1b1b1b" fill-opacity="0.4"');
 return s;
}
const backgrounds=[['Warm', '<rect width="72" height="152" fill="#aca48e"/><path d="M0 0H34V152H0Z" fill="#57584e"/><path d="M25 0H44V152H25Z" fill="#d8d1bd"/>'],['Dark','<rect width="72" height="152" fill="#293b4e"/><path d="M0 30L72 80V120L0 70Z" fill="#657b85"/>'],['Bright','<rect width="72" height="152" fill="#dfc799"/><path d="M0 0L72 100V152L0 52Z" fill="#c76e43"/>']];
for(const scale of [1,3]){
 const w=540,h=90+152*scale;
 let body='<rect width="100%" height="100%" fill="#292929"/>';const layers=[];
 for(let i=0;i<backgrounds.length;i++){
  const [name,bg]=backgrounds[i];
  // Each palette gets its own sheet at 3x; native sheet shows all backgrounds.
  if(scale===3 && i>0) continue;
  for(let variant=0;variant<2;variant++){
   const x=scale===1?20+i*174+variant*80:30+variant*260;
   body+=`<text x="${x}" y="30" fill="white" font-family="Segoe UI" font-size="12">${variant?'Proposal':'Current'}</text>`;
   const stack=[];
   for(const [j,name] of ['upper','lower'].entries()){
    let src=await readFile(`${dir}/1-${name}.svg`,'utf8');if(variant)src=proposal(src);
    await writeFile(`${dir}/${variant?'proposal':'current'}-${name}.svg`,src);
    const key=await sharp(Buffer.from(src)).resize(72,72).png().toBuffer();
    stack.push({input:key,left:0,top:j*80});
   }
   const pair=await sharp(svg(72,152,bg)).composite(stack).png().toBuffer();
   layers.push({input:await sharp(pair).resize(72*scale,152*scale,{kernel:'nearest'}).png().toBuffer(),left:x,top:60});
  }
 }
 await sharp(svg(w,h,body)).composite(layers).png().toFile(`${dir}/comparison-cells-${scale}x.png`);
}
