import { useEffect, useRef, useState, type ReactNode } from 'react';
import { X } from 'lucide-react';
import { api, normalizeError } from './api';

export function safeHostname(value: string) { try { return new URL(value).hostname || value; } catch { return value; } }
export const parseUrls = (text: string) => [...new Set(text.split(/\s+/).filter(Boolean).filter(value => { try { return ['http:', 'https:'].includes(new URL(value).protocol); } catch { return false; } }))];

export function Dialog({ title, children, onClose, footer }: { title:string; children:ReactNode; onClose:()=>void; footer:ReactNode }) {
  const ref = useRef<HTMLDivElement>(null), closeRef = useRef(onClose);
  useEffect(() => { closeRef.current = onClose; }, [onClose]);
  useEffect(() => {
    const focusable = ref.current?.querySelectorAll<HTMLElement>('button,input,select,textarea,[tabindex]'); focusable?.[0]?.focus();
    const key = (event: KeyboardEvent) => { if (event.key === 'Escape') closeRef.current(); if (event.key === 'Tab' && focusable?.length) { const first=focusable[0], last=focusable[focusable.length-1]; if (event.shiftKey && document.activeElement===first) { event.preventDefault(); last.focus(); } else if (!event.shiftKey && document.activeElement===last) { event.preventDefault(); first.focus(); } } };
    document.addEventListener('keydown', key); return () => document.removeEventListener('keydown', key);
  }, []);
  return <div className="dialog-backdrop" role="presentation"><div ref={ref} className="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title"><header><h2 id="dialog-title">{title}</h2><button aria-label="Close" onClick={onClose}><X/></button></header><div className="dialog-body">{children}</div><div className="dialog-footer">{footer}</div></div></div>;
}

function Destination({ value, onChange, onError }: { value:string; onChange:(v:string)=>void; onError:(v:string)=>void }) {
  const browse = async () => { try { const path=await api.chooseDirectory(); if (path) { await api.validateDirectory(path); onChange(path); } } catch (e) { onError(normalizeError(e)); } };
  return <label>Save to<div className="path-field"><input value={value} readOnly placeholder="System Downloads folder"/><button type="button" onClick={() => void browse()}>Browse</button></div></label>;
}

export function AddDialog({ initial, onClose, onAdded, onBatch }: { initial:string; onClose:()=>void; onAdded:()=>void; onBatch:(pattern:string)=>void }) {
  const [url,setUrl]=useState(initial), [filename,setFilename]=useState(''), [filenameEdited,setFilenameEdited]=useState(false), [destination,setDestination]=useState(''), [connections,setConnections]=useState(8), [error,setError]=useState(''), [busy,setBusy]=useState(false), [metadata,setMetadata]=useState('');
  const request=useRef(0), wildcard=url.includes('*');
  useEffect(() => {
    if (wildcard) { setMetadata('Batch pattern detected. Configure its range before adding.'); return; }
    let valid=false; try { valid=['http:','https:'].includes(new URL(url).protocol); } catch { /* invalid */ }
    if (!valid) { setMetadata(''); return; }
    const id=++request.current, timer=window.setTimeout(() => { void api.probe(url).then(result => { if(id!==request.current)return; setMetadata(`${result.total ?? 'Unknown size'} · ${result.ranges?'Range supported':'Single stream'}`); if(!filenameEdited)setFilename(result.filename); }).catch(() => { if(id===request.current)setMetadata('Metadata unavailable'); }); },450);
    return () => { clearTimeout(timer); request.current++; };
  }, [url,filenameEdited,wildcard]);
  const submit=async(start:boolean)=>{setError(''); if(wildcard){onBatch(url);return;} try{const parsed=new URL(url);if(!['http:','https:'].includes(parsed.protocol))throw new Error('Enter a valid HTTP(S) URL');setBusy(true);await api.add(url,start,connections,destination||null);onAdded();onClose();}catch(e){setError(normalizeError(e));}finally{setBusy(false);}};
  return <Dialog title="Add Download" onClose={onClose} footer={<><button onClick={onClose}>Cancel</button>{wildcard?<button className="primary" disabled={busy} onClick={()=>onBatch(url)}>Configure Batch</button>:<><button disabled={busy||!url} onClick={()=>void submit(false)}>Add to queue</button><button className="primary" disabled={busy||!url} onClick={()=>void submit(true)}>Download now</button></>}</>}><form onSubmit={e=>{e.preventDefault();void submit(true)}}><label>URL<input value={url} onChange={e=>setUrl(e.target.value)} autoComplete="off"/></label><div className="validation-slot">{metadata}</div><label>Filename<input value={filename} onChange={e=>{setFilename(e.target.value);setFilenameEdited(true)}} placeholder="Detected automatically"/></label><Destination value={destination} onChange={setDestination} onError={setError}/><label>Connections<input type="number" min="1" max="32" value={connections} onChange={e=>setConnections(Number(e.target.value))}/></label><div className="validation-slot error">{error}</div></form></Dialog>;
}

export function MultipleDialog({urls,onClose,onAdded}:{urls:string[];onClose:()=>void;onAdded:()=>void}) { const[selected,setSelected]=useState(urls),[error,setError]=useState(''); const submit=async(start:boolean)=>{try{await api.addBatch(selected,8,start);onAdded();onClose();}catch(e){setError(normalizeError(e));}}; return <Dialog title="Multiple Downloads" onClose={onClose} footer={<><button onClick={onClose}>Cancel</button><button disabled={!selected.length} onClick={()=>void submit(false)}>Add all to queue</button><button className="primary" disabled={!selected.length} onClick={()=>void submit(true)}>Download all</button></>}><p>{selected.length} valid URLs detected</p><div className="url-list">{selected.map(url=><div key={url}><button aria-label="Remove URL" onClick={()=>setSelected(x=>x.filter(v=>v!==url))}><X/></button><span>{safeHostname(url)}</span><code>{url}</code></div>)}</div><div className="validation-slot error">{error}</div></Dialog>; }

export function BatchDialog({pattern,onClose,onAdded}:{pattern:string;onClose:()=>void;onAdded:()=>void}) {
  const [padding,setPadding]=useState(0),[destination,setDestination]=useState(''),[urls,setUrls]=useState<string[]>([]),[error,setError]=useState(''),[detecting,setDetecting]=useState(false),stars=pattern.split('*').length-1;
  const detect=async()=>{if(stars!==1){setError(stars>1?'Multiple wildcards are not supported.':'Pattern must contain one wildcard.');return;}setDetecting(true);setError('');try{setUrls(await api.discoverBatch(pattern,padding));}catch(e){setUrls([]);setError(normalizeError(e));}finally{setDetecting(false);}};
  useEffect(()=>{void detect();},[]);
  const submit=async(start:boolean)=>{try{await api.addBatch(urls,8,start,destination||null);onAdded();onClose();}catch(e){setError(normalizeError(e));}};
  return <Dialog title="Batch Download" onClose={onClose} footer={<><button onClick={onClose}>Cancel</button><button disabled={detecting} onClick={()=>void detect()}>{detecting?'Checking server…':'Detect files automatically'}</button><button disabled={!urls.length} onClick={()=>void submit(false)}>Add all to queue</button><button className="primary" disabled={!urls.length} onClick={()=>void submit(true)}>Download all</button></>}><label>URL pattern<input value={pattern} readOnly/></label><p>Hyper Get checks the server and stops after the detected sequence ends. You do not need to enter Start or End.</p><label>Number padding<input type="number" min="0" max="12" value={padding} onChange={e=>setPadding(Number(e.target.value))}/></label><Destination value={destination} onChange={setDestination} onError={setError}/><p>{detecting?'Checking candidate links…':`${urls.length} files found`}</p><div className="preview">{urls.slice(0,100).map((url,i)=><div key={url}><b>{i+1}</b><span>{url.split('/').pop()}</span><code>{url}</code></div>)}</div><div className="validation-slot error">{error}</div></Dialog>;
}
