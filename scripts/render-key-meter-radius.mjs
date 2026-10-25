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

const radii=[0,0.5,1,1.5,2];
for(const scale of [1,3]){
 const gap=16,w=20+radii.length*(72*scale+gap),h=80+152*scale;
 let body='<rect width="100%" height="100%" fill="#292929"/>';const layers=[];
 body+='<text x="20" y="24" fill="white" font-family="Segoe UI" font-size="16">Meter background corner radius (native pixels)</text>';
 for(let i=0;i<radii.length;i++){
  const radius=radii[i],x=20+i*(72*scale+gap);
  body+=`<text x="${x}" y="48" fill="white" font-family="Segoe UI" font-size="13">${radius} px</text>`;
  const stack=[];
  for(const [j,name] of ['upper','lower'].entries()){
   let src=proposal(await readFile(`${dir}/1-${name}.svg`,'utf8'));
   const before=src;
   src=src.replace(/height="24" fill="#000000" fill-opacity="0.4"/g,`height="24" rx="${radius*2}" fill="#000000" fill-opacity="0.4"`);
   if(src===before)throw Error('Meter rectangle not found');
   await writeFile(`${dir}/radius-${radius}-${name}.svg`,src);
   stack.push({input:await sharp(Buffer.from(src)).resize(72,72).png().toBuffer(),left:0,top:j*80});
  }
  const pair=await sharp(svg(72,152,backgrounds[0][1])).composite(stack).png().toBuffer();
  layers.push({input:await sharp(pair).resize(72*scale,152*scale,{kernel:'nearest'}).png().toBuffer(),left:x,top:64});
 }
 await sharp(svg(w,h,body)).composite(layers).png().toFile(`${dir}/meter-radius-${scale}x.png`);
}
