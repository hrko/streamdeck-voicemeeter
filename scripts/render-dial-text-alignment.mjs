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
// Measure the visible glyph bottom rather than the 24px text layout box.
const glyph = await sharp(svg(108,24,'<text x="0" y="18" fill="white" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="16" font-weight="600">-6.0 dB</text>')).ensureAlpha().raw().toBuffer();
let glyphBottom = 0;
for(let y=0;y<24;y++) for(let x=0;x<108;x++) if(glyph[(y*108+x)*4+3]>=32) glyphBottom=y+1;
const statusShift = glyphBottom - 22; // Status squares occupy y=15..22.
console.log({glyphBottom,statusShift});
async function render(c,kind,bg,opacity,mode) {
 const frame=c.frames[kind], layers=[];
 for(const item of [...layouts[kind].items].sort((a,b)=>a.zOrder-b.zOrder)) {
  const p=frame[item.key]; if(!p) continue;
  const background=item.key.startsWith("background");
  if(!(background&&opacity!==undefined) && !p.enabled) continue;
  let [x,y,w,h]=item.rect;
  if(mode.startsWith('hidden') && item.key.startsWith('levelMeter')) continue;
  if(mode.startsWith('hidden') && (item.key.startsWith('gainValue') || item.key.startsWith('status'))) y += 8;
  if(mode.endsWith('aligned') && item.key.startsWith('status')) y += statusShift;
  if(mode.endsWith('text-down') && item.key.startsWith('gainValue')) y -= statusShift;
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

const out = "target/dial-text-alignment";
await mkdir(out,{recursive:true});
const rows = [
 {mode:"visible-aligned",label:"Meter ON / previous: indicators moved up 3 px"},
 {mode:"visible-text-down",label:"Meter ON / new: text moved down 3 px"},
 {mode:"hidden-aligned",label:"Meter OFF / previous: indicators moved up 3 px"},
 {mode:"hidden-text-down",label:"Meter OFF / new: text moved down 3 px"},
];
for(const variant of [
 {id:14,name:"voicemeeter",title:"VoiceMeeter / opaque",bg:0},
 {id:15,name:"white-transparent",title:"White / transparent + wallpaper",bg:1},
 {id:16,name:"vm-transparent",title:"VoiceMeeter / transparent + wallpaper",bg:1},
]) for(const scale of [1,2]) {
 const w=860,h=60+rows.length*(100*scale+58);
 let body='<rect width="100%" height="100%" fill="#252525"/>'+text(20,28,variant.title+' — '+scale+'x',20);
 const layers=[];
 for(let i=0;i<rows.length;i++) {
  const row=rows[i],y=70+i*(100*scale+58);
  body+=text(20,y-12,row.label,16);
  for(let kind=0;kind<2;kind++) {
   const input=await render(cases[variant.id],kind,variant.bg,undefined,row.mode);
   if(scale===1) await writeFile(out+'/'+variant.name+'-'+row.mode+'-'+['gain','combo'][kind]+'.png',input);
   layers.push({input:await sharp(input).resize(200*scale,100*scale,{kernel:'nearest'}).png().toBuffer(),left:20+kind*(200*scale+20),top:y});
  }
 }
 await sharp(svg(w,h,body)).composite(layers).png().toFile(out+'/'+variant.name+'-'+scale+'x.png');
}
console.log(out);
