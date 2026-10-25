import assert from "node:assert/strict";
import {readFile} from "node:fs/promises";
import sharp from "sharp";
const fixtures=JSON.parse(await readFile("target/dial-readability/feedback.json","utf8"));
for(const kind of [0,1]){
 const frame=fixtures[0].frames[kind];
 for(const key of kind?["levelMeter","levelMeter1"]:["levelMeter"]){
  const svg=Buffer.from(frame[key].value.split(",")[1],"base64");
  const {data,info}=await sharp(svg).flatten({background:"#000000"}).removeAlpha().raw().toBuffer({resolveWithObject:true});
  const brightness=(x,y)=>Math.max(...data.subarray((y*info.width+x)*3,(y*info.width+x)*3+3));
  for(const y of [1,3]) for(let cell=0;cell<8;cell++){
   const x=2+2*cell;
   assert.ok(brightness(x,y)>80,kind+"/"+key+": lit pixel must remain bright");
   assert.ok(brightness(x+1,y)<10,kind+"/"+key+": a whole dark pixel must separate cells");
  }
  await sharp(svg).resize(info.width*6,info.height*6,{kernel:"nearest"}).png().toFile("target/dial-readability/cell-check-"+kind+"-"+key+".png");
 }
}
console.log("Native-size single and Combo stereo cells retain whole-pixel gaps.");
