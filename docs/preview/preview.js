(() => {
  'use strict';
  const get = id => document.getElementById(id);
  const data = window.BRIOCHE_PREVIEW;
  const lesson = data.lesson;
  const characters = new Map(lesson.cast.map(c => {
    const asset = data.characters.find(item => item.avatarId === c.avatarId);
    return [c.characterId, {...asset, displayName:c.displayName, voice:{locale:c.speechLocale,preferredName:null}}];
  }));
  const vocabulary = new Map(lesson.knowledge.vocabulary.map(v => [v.id, v]));
  const icons = {
    'chevron-down':'<path d="m6 9 6 6 6-6"/>',
    'chevron-right':'<path d="m9 6 6 6-6 6"/>',
    info:'<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/>',
    play: '<path d="m9 5 10 7-10 7Z"/>', pause: '<path d="M8 5v14M16 5v14"/>', stop: '<rect x="6" y="6" width="12" height="12" rx="1"/>',
    volume: '<path d="M11 5 6 9H3v6h3l5 4ZM15 8a6 6 0 0 1 0 8M18 5a10 10 0 0 1 0 14"/>',
    languages: '<path d="M3 5h12M9 3v2M5 5c1 6 4 8 8 10M13 5c-1 6-4 8-9 11M13 21l4-10 4 10M15 17h4"/>',
    bookmark: '<path d="M6 4h12v17l-6-4-6 4Z"/>', check: '<path d="m5 12 4 4L19 6"/>', x: '<path d="m6 6 12 12M18 6 6 18"/>',
    'arrow-right': '<path d="M4 12h16m-6-6 6 6-6 6"/>', 'arrow-up-right': '<path d="M6 18 18 6M6 6h12v12"/>',
    'book-open': '<path d="M12 6c-3-2-6-2-9-1v14c3-1 6-1 9 1 3-2 6-2 9-1V5c-3-1-6-1-9 1Zm0 0v14"/>',
    user: '<circle cx="12" cy="8" r="4"/><path d="M4 21v-2a8 8 0 0 1 16 0v2"/>',
    settings: '<path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3" fill="var(--surface)"/><circle cx="16" cy="17" r="3" fill="var(--surface)"/>',
    leaf: '<path d="M20 3C8 1 1 9 6 16c8 6 16-2 14-13ZM5 20 16 8"/>'
  };
  const icon = name => '<svg class="icon" data-icon-name="' + name + '" viewBox="0 0 24 24" aria-hidden="true" focusable="false">' + icons[name] + '</svg>';
  function hydrateIcons(root = document) { root.querySelectorAll('[data-icon]').forEach(el => { el.innerHTML = icon(el.dataset.icon); }); }
  const escapeHtml = value => String(value).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const reduced = () => matchMedia('(prefers-reduced-motion:reduce)').matches;
  function enter(el, className = 'view-enter') { el.classList.remove(className); if (reduced()) return; void el.offsetWidth; el.classList.add(className); }
  hydrateIcons();
  let toastTimer=null,toastExit=null,toastTrigger=null;
  function scheduleToast() { clearTimeout(toastTimer);toastTimer=setTimeout(hideToast,5500); }
  function hideToast() {
    clearTimeout(toastTimer);clearTimeout(toastExit);
    const toast=get('toast');if(toast.hidden)return;
    if(toast.contains(document.activeElement)){
      const target=toastTrigger?.isConnected&&!toastTrigger.closest('[inert],[hidden]')&&toastTrigger.getClientRects().length?toastTrigger:get('knowledge').classList.contains('is-open')?get('note-close'):null;
      target?.focus({preventScroll:true});
    }
    const finish=()=>{toast.hidden=true;toast.classList.remove('is-leaving');get('toast-message').textContent='';};
    if(reduced())finish();else{toast.classList.add('is-leaving');toastExit=setTimeout(finish,180);}
  }
  function showToast(message) {
    clearTimeout(toastTimer);clearTimeout(toastExit);
    const toast=get('toast');if(!toast.contains(document.activeElement))toastTrigger=document.activeElement;
    toast.classList.remove('is-leaving');toast.hidden=false;get('toast-message').textContent=message;
    if(!toast.matches(':hover')&&!toast.contains(document.activeElement))scheduleToast();
  }
  get('toast-close').addEventListener('click',hideToast);
  get('toast').addEventListener('pointerenter',()=>clearTimeout(toastTimer));
  get('toast').addEventListener('pointerleave',()=>{if(!get('toast').contains(document.activeElement))scheduleToast();});
  get('toast').addEventListener('focusin',()=>clearTimeout(toastTimer));
  get('toast').addEventListener('focusout',event=>{if(!get('toast').contains(event.relatedTarget))scheduleToast();});
  document.addEventListener('pointerdown',event=>{
    if(reduced()||event.button!==0)return;
    const button=event.target.closest('.primary,.secondary,.profile-settings,.icon-button,.review,.reading-tabs button,.source-open');
    if(!button||button.disabled)return;
    const rect=button.getBoundingClientRect();button.style.setProperty('--press-x',(event.clientX-rect.left)+'px');button.style.setProperty('--press-y',(event.clientY-rect.top)+'px');
    button.classList.remove('is-pressed');void button.offsetWidth;button.classList.add('is-pressed');
  });
  document.addEventListener('animationend',event=>{if(event.animationName==='press-wave')event.target.classList.remove('is-pressed');});

  let format = 'dialogue', units = [], selectedTerm = 'expr-je-voudrais', noteTrigger = null, translations = false, playbackRate=1;
  const saved = new Set(), localTranslations = new Set();
  const audio = { mode:null, queue:[], index:0, status:'idle', generation:0, utterance:null, word:null, charIndex:0 };
  function currentBlock() { return lesson.blocks.find(b => b.type === format); }
  function entryCharacter(block, entry) {
    const characterId = format === 'article' ? block.narratorId : block.speakers.find(s => s.id === entry.speakerId).characterId;
    return characters.get(characterId);
  }
  function renderBody() {
    const block = currentBlock();
    units = (block.turns || block.paragraphs).map(entry => ({id:entry.id, text:entry.segments.map(s => s.text).join(''), translation:entry.translationZh, character:entryCharacter(block, entry), entry}));
    const segmenter = new Intl.Segmenter('fr', {granularity:'word'});
    get('reading-body').innerHTML = units.map((unit, unitIndex) => {
      let start = 0;
      const sentence = unit.entry.segments.map(segment => {
        const markup = [...segmenter.segment(segment.text)].map(token => {
          if (!token.isWordLike) return escapeHtml(token.segment);
          const knowledge = segment.vocabularyId || '';
          return '<button type="button" class="word' + (knowledge ? ' known' : '') + '" data-word="' + escapeHtml(token.segment) + '" data-unit="' + unitIndex + '" data-start="' + (start + token.index) + '" data-knowledge="' + knowledge + '" aria-label="朗读 ' + escapeHtml(token.segment) + '">' + escapeHtml(token.segment) + '</button>';
        }).join('');
        start += segment.text.length;
        return markup;
      }).join('');
      if(format==='article')return '<div class="article-paragraph" data-entry="'+unit.id+'"><p class="sentence" lang="fr">'+sentence+'</p><p class="translation" data-translation-entry="'+unit.id+'" hidden>'+escapeHtml(unit.translation)+'</p></div>';
      return '<div class="dialogue-turn" data-entry="'+unit.id+'"><button type="button" class="speaker" data-sentence="'+unitIndex+'" aria-label="'+escapeHtml(unit.character.displayName)+'：翻译并朗读第'+(unitIndex+1)+'句" aria-pressed="false"><img src="'+unit.character.previewAvatarPath+'" alt=""></button><div><div class="sentence" lang="fr">'+sentence+'</div><p class="translation" data-translation-entry="'+unit.id+'" hidden>'+escapeHtml(unit.translation)+'</p></div></div>';
    }).join('');
    get('read-title').textContent = format === 'dialogue' ? lesson.title.fr : 'Une baguette pour le petit-déjeuner';
    get('reading-characters').hidden=format!=='dialogue';
    get('reading-characters').innerHTML=format==='dialogue'?block.speakers.map(s=>{const c=characters.get(s.characterId);return '<li><img src="'+c.previewAvatarPath+'" alt="" width="44" height="44"><div><span lang="fr">'+escapeHtml(c.displayName)+'</span><small>'+escapeHtml(s.labelZh)+'</small></div></li>';}).join(''):'';
    get('reading-context').textContent = format === 'dialogue' ? '社区面包店 · 清晨' : '早餐前的一次采购';
    document.querySelectorAll('[data-reading]').forEach(button => { const active=button.dataset.reading===format; button.setAttribute('aria-selected',String(active)); button.tabIndex=active?0:-1; });
    get('reading-body').setAttribute('aria-labelledby','tab-'+format);
    renderTranslations(); renderAudio(); enter(get('reading-body'));
  }
  function renderTranslations() {
    document.querySelectorAll('[data-translation-entry]').forEach(el => { el.hidden = !(translations || localTranslations.has(el.dataset.translationEntry)); });
    document.querySelectorAll('[data-sentence]').forEach(button=>button.setAttribute('aria-expanded',String(translations||localTranslations.has(units[Number(button.dataset.sentence)].id))));
    get('toggle-translation').setAttribute('aria-checked',String(translations));
  }
  get('toggle-translation').addEventListener('click', () => { translations=!translations; localTranslations.clear(); renderTranslations(); });
  get('reading-body').addEventListener('click', event => {
    const word = event.target.closest('[data-word]');
    if (word) {
      const unit=units[Number(word.dataset.unit)];
      play([{...unit,text:word.dataset.word}], 'word', word);
      if (vocabulary.has(word.dataset.knowledge)) { selectedTerm=word.dataset.knowledge; renderTerm(); openNote(word); }
      return;
    }
    const sentence=event.target.closest('[data-sentence]');
    if (sentence) {
      const unit=units[Number(sentence.dataset.sentence)];
      localTranslations.add(unit.id);renderTranslations();play([unit],'sentence');return;
    }
  });
  document.querySelectorAll('[data-reading]').forEach(button => {
    button.addEventListener('click',()=>{if(format===button.dataset.reading)return;stop();format=button.dataset.reading;localTranslations.clear();renderBody();});
    button.addEventListener('keydown',event=>{if(!['ArrowLeft','ArrowRight','Home','End'].includes(event.key))return;event.preventDefault();const next=event.key==='Home'?'dialogue':event.key==='End'?'article':format==='dialogue'?'article':'dialogue';get('tab-'+next).click();get('tab-'+next).focus();});
  });

  function frenchVoice(character) {
    if (!('speechSynthesis' in window) || !('SpeechSynthesisUtterance' in window)) return null;
    const voices=window.speechSynthesis.getVoices().filter(v=>/^fr(?:-|_)/i.test(v.lang));
    return voices.find(v=>v.name===character?.voice?.preferredName) || voices.find(v=>v.lang==='fr-FR') || voices[0] || null;
  }
  function renderAudio(message) {
    const active=audio.status!=='idle', whole=audio.mode==='whole'&&active;
    get('audio-toolbar').dataset.state=audio.status;
    get('play-whole').setAttribute('aria-label',whole?(audio.status==='paused'?'继续朗读':'暂停朗读'):(format==='dialogue'?'朗读整个对话':'朗读整个文章'));
    const symbol=whole&&audio.status!=='paused'?'pause':'play';
    if(get('play-marker').dataset.symbol!==symbol){get('play-marker').dataset.symbol=symbol;get('play-marker').innerHTML=icon(symbol);}
    const unit=audio.queue[audio.index];
    get('review-flashcard').classList.toggle('is-speaking',Boolean(active&&audio.status==='playing'&&unit?.id===('review-'+reviewQueue[reviewIndex])));
    const text=active?(audio.status==='pending'?'准备语音':audio.status==='paused'?'已暂停':audio.mode==='word'?unit?.text:(unit?.character.displayName+' · '+(audio.mode==='whole'?(audio.index+1)+' / '+audio.queue.length:'整句'))):'';
    get('audio-status').textContent=text;
    if(message)showToast(message);
    const total=audio.queue.reduce((n,u)=>n+u.text.length,0);
    const done=audio.queue.slice(0,audio.index).reduce((n,u)=>n+u.text.length,0)+audio.charIndex;
    const percent=whole&&total?Math.min(100,done/total*100):0;
    get('audio-progress').style.width=percent+'%';
    get('play-marker').style.left=(whole?Math.max(4,Math.min(96,percent)):50)+'%';
    document.querySelectorAll('[data-entry]').forEach(row=>{const current=active&&audio.status!=='pending'&&row.dataset.entry===unit?.id;row.classList.toggle('is-speaking',current&&audio.status==='playing');row.classList.toggle('is-paused',current&&audio.status==='paused');row.querySelector('.speaker')?.setAttribute('aria-pressed',String(current));});
    document.querySelectorAll('.word.is-speaking').forEach(word=>word.classList.remove('is-speaking'));
    if(audio.word&&audio.status==='playing')audio.word.classList.add('is-speaking');
  }
  function stop(message='') {
    audio.generation++;
    if('speechSynthesis' in window)window.speechSynthesis.cancel();
    audio.mode=null;audio.queue=[];audio.index=0;audio.status='idle';audio.utterance=null;audio.word=null;audio.charIndex=0;
    renderAudio(message);
  }
  function speakNext(generation) {
    if(generation!==audio.generation||audio.status==='paused')return;
    if(audio.index>=audio.queue.length){stop();return;}
    const unit=audio.queue[audio.index], voice=frenchVoice(unit.character);
    if(!voice){stop('当前浏览器没有可用的法语语音');return;}
    const utterance=new SpeechSynthesisUtterance(unit.text);
    utterance.lang=voice.lang;utterance.voice=voice;utterance.rate=playbackRate;
    audio.utterance=utterance;audio.status='pending';renderAudio();
    utterance.onstart=()=>{if(generation!==audio.generation)return;if(audio.status!=='paused')audio.status='playing';renderAudio();};
    utterance.onboundary=event=>{
      if(generation!==audio.generation||audio.mode==='word'||audio.status!=='playing')return;
      audio.charIndex=event.charIndex;renderAudio();
      const row=document.querySelector('[data-entry="'+unit.id+'"]');
      if(!row)return;
      const words=[...row.querySelectorAll('[data-start]')];
      const word=words.find(w=>event.charIndex>=Number(w.dataset.start)&&event.charIndex<Number(w.dataset.start)+w.dataset.word.length);
      if(word){audio.word=word;renderAudio();}
    };
    utterance.onend=()=>{if(generation!==audio.generation)return;audio.utterance=null;audio.word=null;audio.charIndex=0;audio.index++;if(audio.status==='paused')return;speakNext(generation);};
    utterance.onerror=event=>{if(generation!==audio.generation)return;stop(event.error==='not-allowed'?'浏览器暂未允许语音播放':'法语语音暂不可用');};
    window.speechSynthesis.speak(utterance);
  }
  function play(queue, mode, word=null) {
    if(audio.status!=='idle'&&mode!=='whole'&&audio.mode===mode&&audio.queue[0]?.id===queue[0]?.id&&audio.queue[0]?.text===queue[0]?.text){stop();return;}
    stop();
    if(!frenchVoice(queue[0]?.character)){renderAudio('当前浏览器没有可用的法语语音');return;}
    audio.mode=mode;audio.queue=queue;audio.word=word;speakNext(audio.generation);
  }
  let suppressPlayUntil=0;
  get('play-whole').addEventListener('click',()=>{
    if(Date.now()<suppressPlayUntil){suppressPlayUntil=0;return;}
    if(audio.mode!=='whole'||audio.status==='idle'){play([...units],'whole');return;}
    if(audio.status==='paused'){audio.status='playing';if(audio.utterance)window.speechSynthesis.resume();else speakNext(audio.generation);}else{window.speechSynthesis.pause();audio.status='paused';}
    renderAudio();
  });
  function applyRate(value) {
    const next=Number(value),changed=next!==playbackRate;playbackRate=next;
    get('settings-rate-label').textContent=value+'×';get('settings-rate').setAttribute('aria-label','朗读速度，'+value+'倍');
    get('audio-rate').querySelectorAll('[data-rate]').forEach(button=>{const active=Number(button.dataset.rate)===next;button.setAttribute('aria-checked',String(active));button.tabIndex=active?0:-1;});
    if(!changed)return;
    if(audio.status==='idle')return;
    audio.generation++;window.speechSynthesis.cancel();audio.utterance=null;audio.word=null;audio.charIndex=0;
    if(audio.status!=='paused'){audio.status='pending';speakNext(audio.generation);}else renderAudio();
  }
  get('audio-rate').addEventListener('click',event=>{const option=event.target.closest('[data-rate]');if(!option)return;applyRate(option.dataset.rate);get('rate-dialog').close();});
  get('audio-rate').addEventListener('keydown',event=>{
    if(!['ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Home','End'].includes(event.key))return;
    event.preventDefault();const options=[...get('audio-rate').querySelectorAll('[data-rate]')],current=options.indexOf(document.activeElement);
    const index=event.key==='Home'?0:event.key==='End'?options.length-1:(Math.max(current,0)+(['ArrowUp','ArrowLeft'].includes(event.key)?-1:1)+options.length)%options.length;
    applyRate(options[index].dataset.rate);options[index].focus();
  });
  get('settings-rate').addEventListener('click',openRate);
  function openRate() { clearPress();if(!get('rate-dialog').open)get('rate-dialog').showModal();get('settings-rate').setAttribute('aria-expanded','true');get('audio-rate').querySelector('[aria-checked="true"]').focus(); }
  get('rate-close').addEventListener('click',()=>get('rate-dialog').close());
  get('rate-dialog').addEventListener('close',()=>{suppressPlayUntil=0;get('settings-rate').setAttribute('aria-expanded','false');});
  get('rate-dialog').addEventListener('click',event=>{if(event.target!==event.currentTarget)return;const r=event.currentTarget.getBoundingClientRect();if(event.clientX<r.left||event.clientX>r.right||event.clientY<r.top||event.clientY>r.bottom)event.currentTarget.close();});
  let pressTimer=null, pressStart=null;
  function clearPress() { clearTimeout(pressTimer);pressTimer=null;pressStart=null; }
  const player=get('play-whole');
  player.addEventListener('pointerdown',event=>{
    if(event.button!==0)return;clearPress();pressStart={x:event.clientX,y:event.clientY};
    pressTimer=setTimeout(()=>{suppressPlayUntil=Date.now()+1000;openRate();},550);
  });
  player.addEventListener('pointermove',event=>{if(pressStart&&Math.hypot(event.clientX-pressStart.x,event.clientY-pressStart.y)>10)clearPress();});
  player.addEventListener('pointerleave',clearPress);window.addEventListener('pointerup',clearPress);window.addEventListener('pointercancel',clearPress);
  window.addEventListener('blur',clearPress);
  player.addEventListener('contextmenu',event=>{event.preventDefault();suppressPlayUntil=Date.now()+1000;openRate();});
  player.addEventListener('keydown',event=>{if(event.key==='F10'&&event.shiftKey){event.preventDefault();openRate();}});
  window.addEventListener('pagehide',()=>stop());

  function renderSaved() { const active=saved.has(selectedTerm);get('save-term').setAttribute('aria-pressed',String(active));get('save-term').innerHTML=icon(active?'check':'bookmark')+'<span id="save-label">'+(active?'已收藏':'收藏表达')+'</span>'; }
  function renderTerm() {
    const term=vocabulary.get(selectedTerm);
    get('term-title').textContent=term.gender==='feminine'?'une '+term.lemma:term.lemma;
    get('term-meaning').textContent=term.meaningZh;
    get('term-detail').textContent=term.partOfSpeech==='phrase'?'常用表达':term.gender==='feminine'?'名词 · 阴性':'词汇';
    get('term-explain').textContent=term.noteZh;
    const dialogue=lesson.blocks.find(b=>b.type==='dialogue');
    const example=dialogue.turns.find(t=>t.segments.some(s=>s.vocabularyId===selectedTerm));
    get('term-example').textContent=example.segments.map(s=>s.text).join('');get('term-example-zh').textContent=example.translationZh;
    renderSaved();enter(get('term-content'),'term-enter');
  }
  get('save-term').addEventListener('click',()=>{saved.has(selectedTerm)?saved.delete(selectedTerm):saved.add(selectedTerm);renderSaved();});
  get('note-audio').addEventListener('click',()=>{const character=characters.get('character-lea');play([{id:'note-expression',text:vocabulary.get(selectedTerm).lemma,character}],'word');});
  function finishClose(restore=true) {
    get('knowledge').classList.remove('is-open');get('knowledge').removeAttribute('role');get('knowledge').removeAttribute('aria-modal');get('knowledge-backdrop').hidden=true;
    document.querySelector('.reading').inert=false;document.querySelector('#view-read .lesson-header').inert=false;document.querySelector('.topbar').inert=false;document.querySelector('.design-note').inert=false;document.body.style.overflow='';
    if(restore&&noteTrigger&&noteTrigger.isConnected)noteTrigger.focus({preventScroll:true});
  }
  function openNote(trigger) {
    if(!matchMedia('(max-width:650px)').matches)return;
    noteTrigger=trigger;get('knowledge').classList.add('is-open');get('knowledge').setAttribute('role','dialog');get('knowledge').setAttribute('aria-modal','true');get('knowledge-backdrop').hidden=false;
    document.querySelector('.reading').inert=true;document.querySelector('#view-read .lesson-header').inert=true;document.querySelector('.topbar').inert=true;document.querySelector('.design-note').inert=true;document.body.style.overflow='hidden';get('note-close').focus({preventScroll:true});
  }
  get('note-close').addEventListener('click',()=>finishClose());get('knowledge-backdrop').addEventListener('click',()=>finishClose());
  document.addEventListener('keydown',event=>{
    if(!get('knowledge').classList.contains('is-open'))return;
    if(event.key==='Escape'){event.preventDefault();finishClose();return;}
    if(event.key==='Tab'){const buttons=[...get('knowledge').querySelectorAll('button')].filter(b=>!b.disabled&&b.getClientRects().length);const first=buttons[0],last=buttons.at(-1);if(event.shiftKey&&document.activeElement===first){event.preventDefault();last.focus();}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}}
  });
  matchMedia('(max-width:650px)').addEventListener('change',()=>finishClose(false));

  const reviewIds=lesson.reviewItemIds.filter(id=>vocabulary.has(id));
  let reviewQueue=[...reviewIds],reviewIndex=0,reviewRevealed=false,reviewResults=[],flashMotion=null;
  const reviewLabel=term=>term.gender==='feminine'?'une '+term.lemma:term.lemma;
  function renderReview(focus=false) {
    flashMotion?.cancel();
    const done=reviewIndex>=reviewQueue.length;
    get('review-stage').hidden=done;get('review-summary').hidden=!done;
    get('review-counter').textContent=done?reviewResults.length+' / '+reviewQueue.length:(reviewIndex+1)+' / '+reviewQueue.length;
    get('review-progress-fill').style.width=(reviewQueue.length?reviewIndex/reviewQueue.length*100:100)+'%';
    if(done){
      get('review-summary-description').textContent='本轮复习了 '+reviewResults.length+' 个表达。';
      [0,1,2].forEach(grade=>get('review-stat-'+grade).textContent=reviewResults.filter(r=>r.grade===grade).length);
      get('review-result-list').innerHTML=reviewResults.map(result=>{const term=vocabulary.get(result.id);return '<li><div><span class="result-expression" lang="fr">'+escapeHtml(reviewLabel(term))+'</span><small>'+escapeHtml(term.meaningZh)+'</small></div><span class="review-result-grade">'+['还不熟','有印象','记住了'][result.grade]+'</span></li>';}).join('');
      get('review-weak').hidden=!reviewResults.some(r=>r.grade<2);
      const hasWeak=!get('review-weak').hidden;
      get('review-restart').className=hasWeak?'text-button':'primary';
      get('review-restart').classList.toggle('summary-main',!hasWeak);
      get('review-session-status').textContent='本轮复习完成，共 '+reviewResults.length+' 个表达';enter(get('review-summary'),'review-question-enter');
      if(focus){get('review-summary-title').tabIndex=-1;get('review-summary-title').focus({preventScroll:true});get('review-summary').scrollIntoView({block:'nearest',behavior:'instant'});}return;
    }
    const term=vocabulary.get(reviewQueue[reviewIndex]),dialogue=lesson.blocks.find(b=>b.type==='dialogue');
    const example=dialogue.turns.find(entry=>entry.segments.some(s=>s.vocabularyId===term.id));
    get('review-kind').textContent=term.partOfSpeech==='phrase'?'常用表达':term.gender==='feminine'?'名词 · 阴性':'日常词汇';
    get('review-prompt').textContent=reviewLabel(term);get('review-meaning').textContent=term.meaningZh;get('review-explanation').textContent=term.noteZh;
    get('review-example-fr').textContent=example?example.segments.map(s=>s.text).join(''):'';get('review-example-zh').textContent=example?.translationZh||'';
    get('review-solution').hidden=!reviewRevealed;get('review-flashcard').setAttribute('aria-expanded',String(reviewRevealed));get('review-flashcard').removeAttribute('aria-describedby');get('review-ratings').hidden=!reviewRevealed;
    get('review-ratings').querySelectorAll('button').forEach(button=>button.disabled=!reviewRevealed);
    get('review-session-status').textContent='第 '+(reviewIndex+1)+' 个，共 '+reviewQueue.length+' 个表达';enter(get('review-stage'),'review-question-enter');
    if(focus){get('review-flashcard').focus({preventScroll:true});get('review-flashcard').scrollIntoView({block:'nearest',behavior:'instant'});}
  }
  get('review-flashcard').addEventListener('click',()=>{
    if(reviewIndex>=reviewQueue.length)return;
    const card=get('review-flashcard'),start=card.getBoundingClientRect().height;flashMotion?.cancel();reviewRevealed=!reviewRevealed;
    get('review-solution').hidden=!reviewRevealed;get('review-ratings').hidden=!reviewRevealed;card.setAttribute('aria-expanded',String(reviewRevealed));
    if(reviewRevealed)card.setAttribute('aria-describedby','review-solution');else card.removeAttribute('aria-describedby');
    get('review-ratings').querySelectorAll('button').forEach(button=>button.disabled=!reviewRevealed);
    const end=card.getBoundingClientRect().height;if(!reduced()&&card.animate)flashMotion=card.animate([{height:start+'px'},{height:end+'px'}],{duration:420,easing:'cubic-bezier(.22,.8,.25,1)'});
    if(reviewRevealed){
      enter(get('review-ratings'),'review-choices-enter');
      const term=vocabulary.get(reviewQueue[reviewIndex]);
      play([{id:'review-'+term.id,text:reviewLabel(term),character:characters.get('character-lea')}],'word');
    }else stop();
  });
  get('review-ratings').addEventListener('click',event=>{
    const button=event.target.closest('[data-grade]');if(!button||!reviewRevealed||reviewIndex>=reviewQueue.length)return;
    stop();reviewResults.push({id:reviewQueue[reviewIndex],grade:Number(button.dataset.grade)});reviewIndex++;reviewRevealed=false;renderReview(true);
  });
  function restartReview(ids) { stop();reviewQueue=[...ids];reviewIndex=0;reviewRevealed=false;reviewResults=[];renderReview(true); }
  get('review-restart').addEventListener('click',()=>restartReview(reviewIds));
  get('review-weak').addEventListener('click',()=>{const weak=reviewResults.filter(r=>r.grade<2).map(r=>r.id);if(weak.length)restartReview(weak);});

  function show(view) {
    clearPress();stop();finishClose(false);if(get('rate-dialog').open)get('rate-dialog').close();
    ['home','read','practice','settings','review'].forEach(v=>get('view-'+v).hidden=v!==view);
    document.querySelectorAll('.preview-controls [data-view]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.view===view)));
    const heading=get(view==='home'?'home-title':view==='read'?'read-title':view==='settings'?'settings-title':view==='review'?'review-page-title':'practice-title');heading.setAttribute('tabindex','-1');heading.focus({preventScroll:true});
    enter(get('view-'+view));document.querySelector('.app').scrollIntoView({block:'start',behavior:'instant'});
  }
  document.querySelectorAll('[data-view]').forEach(b=>b.addEventListener('click',()=>show(b.dataset.view)));
  let reviewMotion=null;
  get('review-card').addEventListener('click',event=>{
    const card=event.currentTarget,start=card.getBoundingClientRect().height,shown=card.getAttribute('aria-expanded')!=='true';
    reviewMotion?.cancel();get('review-answer').hidden=!shown;card.setAttribute('aria-expanded',String(shown));
    if(shown)card.setAttribute('aria-describedby','review-answer');else card.removeAttribute('aria-describedby');
    const end=card.getBoundingClientRect().height;
    if(!reduced()&&card.animate&&Math.abs(end-start)>1)reviewMotion=card.animate([{height:start+'px'},{height:end+'px'}],{duration:320,easing:'cubic-bezier(.22,.8,.25,1)'});
  });
  matchMedia('(prefers-reduced-motion:reduce)').addEventListener('change',event=>{if(event.matches){reviewMotion?.cancel();document.getAnimations?.().forEach(animation=>animation.cancel());document.querySelectorAll('.is-pressed').forEach(button=>button.classList.remove('is-pressed'));}});
  get('source-open').addEventListener('click',()=>{
    stop();const block=lesson.blocks.find(b=>b.type==='dialogue');
    const cast='<ul class="reading-characters">'+block.speakers.map(s=>{const c=characters.get(s.characterId);return '<li><img src="'+c.previewAvatarPath+'" alt=""><div><span lang="fr">'+escapeHtml(c.displayName)+'</span><small>'+escapeHtml(s.labelZh)+'</small></div></li>';}).join('')+'</ul>';
    get('source-transcript').innerHTML=cast+block.turns.map(entry=>{const speaker=block.speakers.find(s=>s.id===entry.speakerId),character=characters.get(speaker.characterId);return '<div class="source-line"><img src="'+character.previewAvatarPath+'" alt="'+escapeHtml(character.displayName)+'"><div class="sentence" lang="fr">'+escapeHtml(entry.segments.map(s=>s.text).join(''))+'</div></div>';}).join('');get('source-dialog').showModal();
  });
  get('source-close').addEventListener('click',()=>get('source-dialog').close());
  get('source-dialog').addEventListener('click',event=>{if(event.target!==get('source-dialog'))return;const rect=event.target.getBoundingClientRect();if(event.clientX<rect.left||event.clientX>rect.right||event.clientY<rect.top||event.clientY>rect.bottom)event.target.close();});

  const form=get('practice-form');
  form.addEventListener('change',()=>{get('submit-answer').disabled=!form.querySelector('input:checked');get('feedback').hidden=true;});
  form.addEventListener('submit',event=>{event.preventDefault();const answer=form.querySelector('input:checked');if(!answer)return;const correct=answer.value==='request';get('feedback').hidden=false;get('feedback').classList.toggle('wrong',!correct);get('feedback-title').textContent=correct?'这是一句礼貌的请求。':'这里是在提出请求。';get('feedback-text').textContent='Je voudrais + 想要的东西，表示“我想要……”。询问价格时，可以说 C’est combien ?。';get('submit-answer').disabled=true;});
  get('retry').addEventListener('click',()=>{form.reset();get('feedback').hidden=true;get('submit-answer').disabled=true;form.querySelector('input').focus();});
  renderReview();renderBody();renderTerm();
})();
