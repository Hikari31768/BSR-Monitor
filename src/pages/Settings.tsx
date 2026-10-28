import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, SlidersHorizontal, RotateCcw, Save, Loader2, RotateCw } from 'lucide-react';

interface ConcurrencySettings {
  startupPlayers: number;
  startupPlaylists: number;
  mapInfo: number;
  addPlayerPages: number;
}

interface AppSettings {
  concurrency: ConcurrencySettings;
  dataDirectory: string;
  currentDataDirectory: string;
  defaultDataDirectory: string;
  restartRequired: boolean;
  migrationError: string | null;
}

const defaults: ConcurrencySettings = {
  startupPlayers: 5, startupPlaylists: 5, mapInfo: 5, addPlayerPages: 5,
};

const concurrencyRows: { key: keyof ConcurrencySettings; label: string; description: string }[] = [
  { key: 'startupPlayers', label: '启动 · 刷新玩家数据', description: '同时更新的玩家数量' },
  { key: 'startupPlaylists', label: '启动 · 刷新歌单数据', description: '同时更新的歌单数量' },
  { key: 'mapInfo', label: '启动 · 谱面信息补充', description: '同时进行的谱面信息补充任务数量' },
  { key: 'addPlayerPages', label: '添加玩家 · 拉取成绩', description: '同一玩家同时请求的成绩页数，每页 100 条' },
];

export default function Settings() {
  const [saved, setSaved] = useState<AppSettings | null>(null);
  const [concurrency, setConcurrency] = useState<ConcurrencySettings>(defaults);
  const [directory, setDirectory] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  async function loadSettings() {
    setLoading(true);
    setError('');
    try {
      const settings = await invoke<AppSettings>('get_settings');
      setSaved(settings);
      setConcurrency(settings.concurrency);
      setDirectory(settings.dataDirectory);
    } catch (e) {
      setError(`读取设置失败：${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { void loadSettings(); }, []);

  const dirty = saved !== null && (directory !== saved.dataDirectory
    || concurrencyRows.some(row => concurrency[row.key] !== saved.concurrency[row.key]));

  async function chooseDirectory() {
    try {
      const selected = await open({ directory: true, multiple: false, title: '选择空文件夹作为数据目录' });
      if (typeof selected === 'string') {
        setDirectory(selected);
        setMessage('');
        setError('');
      }
    } catch (e) {
      setError(`选择目录失败：${e}`);
    }
  }

  async function saveSettings() {
    setSaving(true);
    setError('');
    setMessage('');
    try {
      const settings = await invoke<AppSettings>('save_settings', { concurrency, dataDirectory: directory });
      setSaved(settings);
      setConcurrency(settings.concurrency);
      setDirectory(settings.dataDirectory);
      setMessage(settings.restartRequired ? '设置已保存，重启应用后生效。' : '设置已保存。');
    } catch (e) {
      setError(`保存失败：${e}`);
    } finally {
      setSaving(false);
    }
  }

  async function restart() {
    setRestarting(true);
    setError('');
    try {
      await invoke('restart_app');
    } catch (e) {
      setError(String(e));
      setRestarting(false);
    }
  }

  if (loading) return <div className="flex items-center gap-2 text-bs-text-secondary"><Loader2 className="animate-spin" size={18} />正在读取设置…</div>;
  if (!saved) return <div role="alert" className="text-red-400">{error}<button onClick={loadSettings} className="ml-4 underline">重试</button></div>;

  const buttonClass = 'inline-flex items-center justify-center gap-2 px-4 py-2 rounded-xl border border-bs-border text-sm text-bs-text-secondary hover:bg-bs-bg-tertiary hover:text-bs-text-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed';

  return (
    <div className="max-w-4xl mx-auto animate-fade-in pb-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-bs-text-primary">设置</h1>
        <p className="text-sm text-bs-text-muted mt-2">调整任务并发和数据保存位置，保存后重启应用生效。</p>
      </div>

      <section aria-labelledby="concurrency-heading" className="bg-bs-bg-secondary border border-bs-border rounded-2xl mb-6 overflow-hidden">
        <div className="px-6 py-5 border-b border-bs-border flex items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <SlidersHorizontal size={20} className="text-bs-accent-primary" aria-hidden="true" />
            <div>
              <h2 id="concurrency-heading" className="font-semibold">并发线程</h2>
            </div>
          </div>
          <button className={buttonClass} disabled={saving || restarting} onClick={() => { setConcurrency({ ...defaults }); setMessage(''); }}>
            <RotateCcw size={14} aria-hidden="true" />恢复默认
          </button>
        </div>
        <div className="px-6 divide-y divide-bs-border">
          {concurrencyRows.map(row => (
            <div key={row.key} className="py-5 flex flex-col sm:flex-row sm:items-center gap-5 sm:gap-8">
              <div className="flex-1 min-w-0">
                <label htmlFor={row.key} className="text-sm font-medium text-bs-text-primary">{row.label}</label>
                <p id={`${row.key}-description`} className="text-xs text-bs-text-muted mt-1">{row.description}</p>
              </div>
              <div className="w-full sm:w-60 flex items-center gap-4">
                <input id={row.key} type="range" min={1} max={5} step={1} value={concurrency[row.key]}
                  disabled={saving || restarting} aria-describedby={`${row.key}-description`}
                  onChange={event => { setConcurrency(prev => ({ ...prev, [row.key]: Number(event.target.value) })); setMessage(''); }}
                  className="settings-slider w-full"
                  style={{ background: `linear-gradient(to right, var(--color-accent-primary) ${(concurrency[row.key] - 1) * 25}%, var(--color-border) ${(concurrency[row.key] - 1) * 25}%)` }} />
                <output htmlFor={row.key} style={{ backgroundColor: 'rgba(62, 207, 142, 0.1)' }} className="text-bs-accent-primary tabular-nums font-semibold w-9 h-9 rounded-lg flex items-center justify-center shrink-0">{concurrency[row.key]}</output>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section aria-labelledby="directory-heading" className="bg-bs-bg-secondary border border-bs-border rounded-2xl p-6">
        <div className="flex items-center gap-3 mb-5">
          <FolderOpen size={20} className="text-bs-accent-primary" aria-hidden="true" />
          <h2 id="directory-heading" className="font-semibold">数据目录位置</h2>
        </div>
        <div className="bg-bs-bg-primary border border-bs-border rounded-xl p-4 break-all text-sm text-bs-text-secondary" aria-label="数据目录">{directory}</div>
        {directory !== saved.currentDataDirectory && <p className="text-xs text-bs-text-muted mt-3 break-all">当前使用：{saved.currentDataDirectory}</p>}
        <div className="mt-5 flex flex-wrap gap-3">
          <button className={buttonClass} disabled={saving || restarting} onClick={chooseDirectory}><FolderOpen size={16} aria-hidden="true" />更改目录</button>
          <button className={buttonClass} onClick={() => invoke('open_data_dir').catch(e => setError(`打开目录失败：${e}`))}>打开当前目录</button>
          {directory !== saved.currentDataDirectory && <button className={buttonClass} disabled={saving || restarting} onClick={() => { setDirectory(saved.currentDataDirectory); setMessage(''); }}>取消目录更改</button>}
        </div>
      </section>

      {saved.migrationError && <p role="alert" className="mt-5 text-sm text-amber-400">{saved.migrationError}</p>}
      {error && <p role="alert" className="mt-5 text-sm text-red-400">{error}</p>}
      {message && <p role="status" className="mt-5 text-sm text-bs-accent-primary">{message}</p>}
      <div className="mt-6 flex items-center justify-end gap-3 flex-wrap">
        {saved.restartRequired && <button className={buttonClass} disabled={dirty || saving || restarting} onClick={restart}>
          {restarting ? <Loader2 size={16} className="animate-spin" /> : <RotateCw size={16} />}重启应用
        </button>}
        <button disabled={!dirty || saving || restarting} onClick={saveSettings}
          className="inline-flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-medium bg-bs-accent-primary text-bs-bg-primary hover:brightness-110 transition disabled:opacity-50 disabled:cursor-not-allowed">
          {saving ? <Loader2 size={16} className="animate-spin" /> : <Save size={16} />}保存设置
        </button>
      </div>
    </div>
  );
}
