import { useEffect, useState } from 'react';
import { ChevronDown, ChevronUp, ClipboardPaste, FolderOpen, Settings as Gear, X } from 'lucide-react';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { api } from './api';
import { BatchDialog, Dialog } from './dialogs';
import type { Settings } from './types';
import './quick.css';

export function QuickRoot() {
  const [url, setUrl] = useState('');
  const [error, setError] = useState('');
  const [settings, setSettings] = useState<Settings | null>(null);
  const [batch, setBatch] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [destination, setDestination] = useState('');
  useEffect(() => { void api.settings().then(async value => {
    if (!value.default_download_directory) { value.default_download_directory = await api.systemDownloads(); await api.updateSettings(value); }
    setSettings(value); setDestination(value.default_download_directory);
  }); }, []);
  if (!settings) return null;
  const submit = async (start: boolean) => { setError(''); try {
    const parsed = new URL(url); if (!['http:', 'https:'].includes(parsed.protocol)) throw new Error('Enter a valid HTTP(S) URL');
    await api.add(url, start, settings.default_connections_per_file, destination || null); setUrl('');
  } catch (reason) { setError(String(reason)); } };
  const toggle = async () => { const next = { ...settings, quick_download_bar_expanded: !settings.quick_download_bar_expanded }; setSettings(next); await api.updateSettings(next); };
  return <>
    <button className="settings-launch" onClick={() => setShowSettings(true)}><Gear/><span>Settings</span></button>
    <div className={`quick-bar ${settings.quick_download_bar_expanded ? 'expanded' : 'collapsed'}`}>
      <button aria-label="Toggle Quick Download" onClick={toggle}>{settings.quick_download_bar_expanded ? <ChevronDown/> : <ChevronUp/>}</button><b>Quick Download</b>
      {settings.quick_download_bar_expanded && <div className="quick-fields">
        <input aria-label="Quick Download URL" value={url} onChange={event => setUrl(event.target.value)} placeholder="Paste or enter URL"/>
        <button onClick={async () => setUrl(await readText())}><ClipboardPaste/>Paste</button>
        <button onClick={async () => { const path = await api.chooseDirectory(); if (path) { await api.validateDirectory(path); setDestination(path); } }}><FolderOpen/>Destination</button>
        <button disabled={!url || url.includes('*')} onClick={() => submit(false)}>Queue</button>
        <button className="primary" disabled={!url} onClick={() => url.includes('*') ? setBatch(true) : submit(true)}>{url.includes('*') ? 'Configure Batch' : 'Download'}</button>
        <button aria-label="Clear" onClick={() => setUrl('')}><X/></button>{error && <span className="quick-error">{error}</span>}
      </div>}
    </div>
    {batch && <BatchDialog pattern={url} onClose={() => setBatch(false)} onAdded={() => { setBatch(false); setUrl(''); }}/>} 
    {showSettings && <SettingsDialog settings={settings} close={() => setShowSettings(false)} saved={setSettings}/>} 
  </>;
}

function SettingsDialog({ settings, close, saved }: { settings: Settings; close: () => void; saved: (s: Settings) => void }) {
  const [value, setValue] = useState(settings); const [error, setError] = useState('');
  const save = async () => { try { await api.validateDirectory(value.default_download_directory); await api.updateSettings(value); saved(value); close(); } catch (reason) { setError(String(reason)); } };
  return <Dialog title="Settings" onClose={close} footer={<><button onClick={close}>Cancel</button><button className="primary" onClick={save}>Save settings</button></>}>
    <h3>General</h3><label>Theme<select value={value.theme} onChange={e => setValue({...value, theme:e.target.value})}><option>system</option><option>dark</option><option>light</option></select></label>
    <label><input type="checkbox" checked={value.confirm_before_delete} onChange={e => setValue({...value,confirm_before_delete:e.target.checked})}/> Confirm before deleting files</label>
    <label><input type="checkbox" checked={value.restore_unfinished_downloads} onChange={e => setValue({...value,restore_unfinished_downloads:e.target.checked})}/> Restore unfinished downloads</label>
    <label><input type="checkbox" checked={value.auto_start_next} onChange={e => setValue({...value,auto_start_next:e.target.checked})}/> Automatically start queued downloads</label>
    <h3>Download location</h3><label>Default directory<div className="path-field"><input value={value.default_download_directory} readOnly/><button onClick={async()=>{const path=await api.chooseDirectory();if(path)setValue({...value,default_download_directory:path,last_selected_directory:path})}}>Browse</button></div></label>
    <button onClick={async()=>setValue({...value,default_download_directory:await api.systemDownloads()})}>Reset to system Downloads</button>
    <label><input type="checkbox" checked={value.ask_where_to_save} onChange={e=>setValue({...value,ask_where_to_save:e.target.checked})}/> Ask where to save each download</label>
    <label><input type="checkbox" checked={value.remember_last_directory} onChange={e=>setValue({...value,remember_last_directory:e.target.checked})}/> Remember last selected directory</label>
    <label><input type="checkbox" checked={value.create_category_subfolders} onChange={e=>setValue({...value,create_category_subfolders:e.target.checked})}/> Create category subfolders</label>
    <h3>Wildcard</h3><label>Batch behavior<select value={value.wildcard_batch_behavior} onChange={e=>setValue({...value,wildcard_batch_behavior:e.target.value})}><option value="preview">Always show preview</option><option value="queue">Add generated links to queue</option><option value="start">Start generated links immediately</option></select></label>
    <label><input type="checkbox" checked={value.wildcard_auto_start} onChange={e=>setValue({...value,wildcard_auto_start:e.target.checked})}/> Start wildcard batch after confirmation</label><div className="validation-slot error">{error}</div>
  </Dialog>;
}
