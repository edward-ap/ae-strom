export function spectrumSegments(height, counts, weights) {
  if(weights) {
    const rows=counts.reduce((a,b)=>a+b,0);
    const gap=Math.min(1,height/(2*rows));
    const activeWeight=counts.reduce((sum,n,i)=>sum+n*weights[i+1],0);
    const unit=(height-gap*rows)/(weights[0]+activeWeight);
    const base=weights[0]*unit,segments=[];
    let y=height-base,filled=0;
    for(let zone=0;zone<3;zone++)for(let row=0;row<counts[zone];row++) {
      const size=weights[zone+1]*unit;y-=gap+size;filled+=weights[zone+1];
      segments.push({y:Math.max(0,y),height:size,zone,threshold:Math.min(1,filled/activeWeight)});
    }
    segments[segments.length-1].threshold=1;
    return {base,segments};
  }
  const base = Math.min(4,height/6), gap = Math.min(2,height/60);
  const third = (height-base-gap)/3, segments=[];
  for(let zone=0;zone<3;zone++) {
    // Very small displays still have nonnegative, bounded geometry.
    const space=Math.min(gap,third/(counts[zone]*2));
    const size=Math.max(0,(third-space*(counts[zone]-1))/counts[zone]);
    for(let row=0;row<counts[zone];row++) segments.push({
      y:Math.max(0,height-base-gap-zone*third-row*(size+space)-size),
      height:size, zone, threshold:(zone+(row+1)/counts[zone])/3
    });
  }
  return {base,segments};
}

export function spectrumInteger(value,name,min,max) {
  if(!Number.isInteger(value)||value<min||value>max)throw new RangeError(`${name} must be an integer in ${min}..${max}.`);
  return value;
}

export function spectrumRowCounts(rows) {
  spectrumInteger(rows,'rows',3,96);
  const counts=[1,1,1],weights=[6,4,3];
  for(let total=3;total<rows;total++) {
    let next=-1;
    for(let i=0;i<3;i++)if(counts[i]<32&&(next<0||counts[i]/weights[i]<counts[next]/weights[next]))next=i;
    counts[next]++;
  }
  return counts;
}
