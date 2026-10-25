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
  if(mode !== 'visible' && item.key.startsWith('levelMeter')) continue;
  if(mode === 'compact' && (item.key.startsWith('gainValue') || item.key.startsWith('status'))) y += 8;
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
 return sharp(svg(200,100,'<rect width="200" height="100" fill="black"/>')).composite(layers).png().toBuffer();
}

const out = "target/dial-meter-hidden-fixed-fader";
await mkdir(out,{recursive:true});
const rows = [
 {mode:"visible",label:"Meter shown / fader y=74"},
 {mode:"hidden",label:"Current: meter hidden / gap 10 px"},
 {mode:"compact",label:"Proposal: value + status y=48 / gap 2 px / fader stays y=74"},
];
for(const scale of [1,2]) {
 const w=860,h=60+rows.length*(100*scale+58);
 let body='<rect width="100%" height="100%" fill="#252525"/>'+text(20,28,'Hidden meter spacing — '+scale+'x',20);
 const layers=[];
 for(let i=0;i<rows.length;i++) {
  const row=rows[i],y=70+i*(100*scale+58);
  body+=text(20,y-12,row.label,16);
  for(let kind=0;kind<2;kind++) {
   const input=await render(cases[11],kind,0,undefined,row.mode);
   if(scale===1) await writeFile(out+'/'+row.mode+'-'+['gain','combo'][kind]+'.png',input);
   layers.push({input:await sharp(input).resize(200*scale,100*scale,{kernel:'nearest'}).png().toBuffer(),left:20+kind*(200*scale+20),top:y});
  }
 }
 await sharp(svg(w,h,body)).composite(layers).png().toFile(out+'/comparison-'+scale+'x.png');
}
console.log(out);
