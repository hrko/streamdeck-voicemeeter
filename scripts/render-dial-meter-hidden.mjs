/**
 * Run after: mise exec rust -- cargo run --locked --example dial_readability
 * Native host text is approximated; SVG components and rectangles use production output.
 */
import sharp from "sharp";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const dir = "target/dial-meter-hidden";
await mkdir(dir, { recursive: true });
const cases = JSON.parse(await readFile("target/dial-readability/feedback.json", "utf8"));
const layouts = await Promise.all(["gain", "combo"].map(async n => JSON.parse(await readFile("assets/layouts/" + n + ".json", "utf8"))));
const esc = s => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
const svg = (w,h,body) => Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="'+w+'" height="'+h+'">'+body+'</svg>');
const text = (x,y,s,size=18,color="#e6eaf0") => '<text x="'+x+'" y="'+y+'" fill="'+color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+size+'">'+esc(s)+'</text>';
function wallpaper(type) {
 if(type===0) return '<rect width="200" height="100" fill="#18263b"/><path d="M0 80L85 12L200 100" fill="#26354c"/>';
 if(type===1) return '<defs><linearGradient id="wall"><stop stop-color="#ffe7a0"/><stop offset="1" stop-color="#f57838"/></linearGradient></defs><rect width="200" height="100" fill="url(#wall)"/>';
 return '<rect width="200" height="100" fill="#18374e"/>'+Array.from({length:12},(_,i)=>'<path d="M'+(i*24-100)+' 0l100 100h12L'+(i*24-88)+' 0z" fill="'+(i%2?"#efaf51":"#cbe7ee")+'"/>').join("");
}
async function render(c,kind,variant,opacity) {
 const frame=structuredClone(c.frames[kind]), layers=[];
 const layout=structuredClone(layouts[kind]);
 for(const item of layout.items) {
  if(variant && item.key.startsWith("levelMeter")) frame[item.key].enabled=false;
  if(variant!==2) continue;
  const offset=item.key.endsWith("1")?100:0, left=kind===0?76:8+offset, width=kind===0?108:84;
  if(item.key.startsWith("gainValue")) {item.rect=[left,34,width,32];item.font.size=24;}
  if(item.key.startsWith("gainSlider")) item.rect=[left,70,width,12];
  if(item.key.startsWith("status")) {
   item.rect=[left,86,84,9];
   const original=Buffer.from(frame[item.key].value.split(",")[1],"base64").toString();
   let x=0;
   const body=[...original.matchAll(/<rect\b[^>]*\/>/g)].map(m=>{
    let r=m[0]; const outlined=r.includes('stroke-width="1"'), small=Number(r.match(/width="([^"]+)"/)[1])<=5;
    const pad=outlined?.5:0;
    r=r.replace(/x="[^"]+"/,'x="'+(x+pad)+'"').replace(/y="[^"]+"/,'y="'+((small?2:1)+pad)+'"');
    x+=small?6:9; return r;
   }).join("");
   frame[item.key].value="data:image/svg+xml;base64,"+svg(84,9,body).toString("base64");
  }
 }
 for(const item of [...layout.items].sort((a,b)=>a.zOrder-b.zOrder)) {
  const p=frame[item.key]; if(!p) continue;
  const background=item.key.startsWith("background");
  if(!(background&&opacity!==undefined) && !p.enabled) continue;
  const [x,y,w,h]=item.rect;
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
   input=await sharp(svg(w,h,mask+'<text x="'+(right?w:0)+'" y="'+(item.font.size===24?25:18)+'" text-anchor="'+(right?"end":"start")+'" fill="'+p.color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+item.font.size+'" font-weight="'+item.font.weight+'"'+(fade?' mask="url(#m)"':"")+'>'+esc(p.value)+'</text>')).png().toBuffer();
  }
  layers.push({input,left:x,top:y});
 }
 return sharp(svg(200,100,'<rect width="200" height="100" fill="#000000"/>')).composite(layers).png().toBuffer();
}


// Preview only: production SVG components, proposed layout rectangles and status reflow.
// Official $B1 value font: size 24, weight 600.
// https://docs.elgato.com/streamdeck/sdk/guides/dials/
const width=688,height=640,layers=[];
let body='<rect width="100%" height="100%" fill="#252525"/>'+text(16,25,"Meter visibility / layout preview",20);
const headings=["Meter ON / current","Meter OFF / current","Meter OFF / proposed"];
for(let col=0;col<3;col++) body+=text(16+col*224,53,headings[col],14);
for(let row=0;row<4;row++){
 const kind=row%2, c=structuredClone(cases[row<2?11:9]);
 // Use an opaque black background for an uncluttered geometry comparison.
 const y=92+row*136;
 body+=text(16,y-9,(row<2?"Default":"Monochrome")+" / "+(kind?"Combo":"Single"),13);
 for(let col=0;col<3;col++){
  const input=await render(c,kind,col,1);
  await writeFile(dir+"/"+row+"-"+col+".png",input);
  layers.push({input,left:16+224*col,top:y});
 }
}
body+=text(16,633,"Native text is approximated. Proposed: 24px values, bar y=70, status row y=86.",12);
const result=await sharp(svg(width,height,body)).composite(layers).png().toBuffer();
await writeFile(dir+"/comparison-1x.png",result);
await sharp(result).resize(width*2,height*2,{kernel:"nearest"}).png().toFile(dir+"/comparison-2x.png");
console.log(dir+"/comparison-1x.png");
