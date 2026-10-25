/**
 * Run after: mise exec rust -- cargo run --locked --example dial_readability
 * Native host text is approximated; SVG components and rectangles use production output.
 */
import sharp from "sharp";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const dir = "target/dial-readability";
await mkdir(dir, { recursive: true });
const cases = JSON.parse(await readFile(dir + "/feedback.json", "utf8"));
const layouts = await Promise.all(["gain", "combo"].map(async n => JSON.parse(await readFile("assets/layouts/" + n + ".json", "utf8"))));
const esc = s => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
const svg = (w,h,body) => Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="'+w+'" height="'+h+'">'+body+'</svg>');
const text = (x,y,s,size=18,color="#e6eaf0") => '<text x="'+x+'" y="'+y+'" fill="'+color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+size+'">'+esc(s)+'</text>';
function wallpaper(type) {
 if(type===3) return '<rect width="200" height="100" fill="black"/>';
 if(type===0) return '<rect width="200" height="100" fill="#18263b"/><path d="M0 80L85 12L200 100" fill="#26354c"/>';
 if(type===1) return '<defs><linearGradient id="wall"><stop stop-color="#ffe7a0"/><stop offset="1" stop-color="#f57838"/></linearGradient></defs><rect width="200" height="100" fill="url(#wall)"/>';
 return '<rect width="200" height="100" fill="#18374e"/>'+Array.from({length:12},(_,i)=>'<path d="M'+(i*24-100)+' 0l100 100h12L'+(i*24-88)+' 0z" fill="'+(i%2?"#efaf51":"#cbe7ee")+'"/>').join("");
}
async function render(c,kind,bg,opacity,mode) {
 const frame=c.frames[kind], layers=[];
 for(const item of [...layouts[kind].items].sort((a,b)=>a.zOrder-b.zOrder)) {
  const p=frame[item.key]; if(!p) continue;
  const background=item.key.startsWith("background");
  if(!(background&&opacity!==undefined) && !p.enabled) continue;
  let [x,y,w,h]=item.rect;


  let input;
  if(item.type==="pixmap") {
   const source=Buffer.from(p.value.split(",")[1],"base64");
   const alpha=background?(opacity??p.opacity??1):1;
   if(alpha===0) continue;
   const raw=await sharp(source).resize(w,h).ensureAlpha().raw().toBuffer();
   for(let i=3;i<raw.length;i+=4) raw[i]=Math.round(raw[i]*alpha);
   input=await sharp(raw,{raw:{width:w,height:h,channels:4}}).png().toBuffer();
  } else {
   const right=item.alignment==="right";
   const fade=item["text-overflow"]==="fade";
   const mask=fade?'<defs><linearGradient id="fade"><stop offset="0.75" stop-color="white"/><stop offset="1" stop-color="black"/></linearGradient><mask id="m"><rect width="'+w+'" height="'+h+'" fill="url(#fade)"/></mask></defs>':"";
   input=await sharp(svg(w,h,mask+'<text x="'+(right?w:0)+'" y="18" text-anchor="'+(right?"end":"start")+'" fill="'+p.color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+item.font.size+'" font-weight="'+item.font.weight+'"'+(fade?' mask="url(#m)"':"")+'>'+esc(p.value)+'</text>')).png().toBuffer();
  }
  layers.push({input,left:x,top:y});
 }
 return sharp(svg(200,100,wallpaper(bg))).composite(layers).png().toBuffer();
}

const out="target/combo-selection-subtle";
await mkdir(out,{recursive:true});
const candidates=[
 ["E","Small dot / 4 px",'<circle cx="50" cy="5" r="2"/>'],
 ["F","Small triangle / 6 x 3 px",'<path d="M47 3.5H53L50 6.5Z"/>'],
 ["G","Short bottom line / 16 x 2 px",'<rect x="42" y="93" width="16" height="2" rx="1"/>'],
 ["H","Small label-side mark / 2 x 10 px",'<rect x="3" y="18" width="2" height="10" rx="1"/>'],
];
function marker(shape,side){
 return svg(200,100,'<g transform="translate('+side*100+' 0)" stroke-linejoin="round"><g fill="#b5d7e5" stroke="black" stroke-opacity="0.55" stroke-width="1">'+shape+'</g><g fill="#b5d7e5" stroke="none">'+shape+'</g></g>');
}
for(const variant of [
 {id:11,name:"default",title:"Default / black background",bg:3},
 {id:14,name:"voicemeeter",title:"VoiceMeeter / opaque",bg:0},
 {id:16,name:"transparent",title:"VoiceMeeter / transparent wallpaper",bg:1},
]) {
 const base=await render(cases[variant.id],1,variant.bg,undefined,"visible");
 for(const scale of [1,2]){
  const width=40+420*scale,height=90+candidates.length*(100*scale+44);
  let body='<rect width="100%" height="100%" fill="#252525"/>'+text(20,26,variant.title+' / '+scale+'x',20);
  body+=text(20,52,'Left selected',14)+text(20+220*scale,52,'Right selected',14);
  const layers=[];
  for(let i=0;i<candidates.length;i++){
   const [id,label,shape]=candidates[i], y=86+i*(100*scale+44);
   body+=text(20,y-10,id+' — '+label,15);
   for(let side=0;side<2;side++){
    const marked=await sharp(base).composite([{input:marker(shape,side),left:0,top:0}]).png().toBuffer();
    if(scale===1) await writeFile(out+'/'+variant.name+'-'+id+'-'+side+'.png',marked);
    layers.push({input:await sharp(marked).resize(200*scale,100*scale,{kernel:'nearest'}).png().toBuffer(),left:20+220*scale*side,top:y});
   }
  }
  await sharp(svg(width,height,body)).composite(layers).png().toFile(out+'/'+variant.name+'-'+scale+'x.png');
 }
}
console.log(out);
