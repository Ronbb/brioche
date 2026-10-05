(() => {
  const root=document.documentElement,track=document.createElement('div'),thumb=document.createElement('div');
  track.className='page-scrollbar';thumb.className='page-scrollbar-thumb';track.append(thumb);
  track.setAttribute('role','scrollbar');track.setAttribute('aria-label','页面滚动');track.setAttribute('aria-orientation','vertical');track.setAttribute('aria-valuemin','0');track.tabIndex=0;
  if(!root.id)root.id='page-scroll-viewport';track.setAttribute('aria-controls',root.id);
  document.body.append(track);root.classList.add('overlay-scroll');
  let timer,frame,drag=null;
  function update(){
    frame=null;const max=Math.max(0,root.scrollHeight-innerHeight),height=track.clientHeight;
    track.hidden=max===0;track.setAttribute('aria-valuemax',String(max));track.setAttribute('aria-valuenow',String(Math.round(scrollY)));
    const size=Math.min(height,Math.max(32,height*innerHeight/root.scrollHeight));thumb.style.height=size+'px';thumb.style.transform='translateY('+(max?scrollY/max*(height-size):0)+'px)';
  }
  function reveal(){track.classList.add('visible');clearTimeout(timer);timer=setTimeout(()=>{if(!drag)track.classList.remove('visible');},1000);}
  function schedule(){if(!frame)frame=requestAnimationFrame(update);}
  addEventListener('scroll',()=>{schedule();reveal();},{passive:true});addEventListener('resize',schedule);
  new ResizeObserver(schedule).observe(document.body);
  track.addEventListener('pointerdown',event=>{
    if(event.button!==0)return;update();const rect=thumb.getBoundingClientRect();
    if(event.clientY<rect.top||event.clientY>rect.bottom){scrollBy(0,(event.clientY<rect.top?-1:1)*innerHeight*.85);return;}
    drag={y:event.clientY,scroll:scrollY};track.setPointerCapture(event.pointerId);reveal();event.preventDefault();
  });
  track.addEventListener('pointermove',event=>{if(drag){const travel=track.clientHeight-thumb.clientHeight;if(travel>0)scrollTo(0,drag.scroll+(event.clientY-drag.y)/travel*(root.scrollHeight-innerHeight));}});
  const finish=()=>{drag=null;reveal();};track.addEventListener('pointerup',finish);track.addEventListener('pointercancel',finish);track.addEventListener('lostpointercapture',finish);
  track.addEventListener('keydown',event=>{
    const steps={ArrowDown:48,ArrowUp:-48,PageDown:innerHeight*.85,PageUp:-innerHeight*.85};
    if(event.key in steps){event.preventDefault();scrollBy(0,steps[event.key]);}
    else if(event.key==='Home'||event.key==='End'){event.preventDefault();scrollTo(0,event.key==='Home'?0:root.scrollHeight);}
  });
  update();
})();
