import { useState, useCallback, useMemo, useEffect } from 'react';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './App.css';

// ---- Types matching Rust backend ----
interface ImageEntry {
  path: string;
  filename: string;
}

interface DateEntry {
  path: string;
  filename: string;
  date: string | null;
  date_source: string | null;
  is_outlier: boolean;
}

interface InferResult {
  date: string | null;
  count: number;
  total: number;
  has_conflict: boolean;
  conflict_message: string | null;
  date_entries: DateEntry[];
}

interface QueueItem {
  folderPath: string;
  folderName: string;
  status: string;
}

interface RenameEntry {
  timestamp: string;
  old_path: string;
  new_path: string;
  old_name: string;
  new_name: string;
}

type Feedback = { type: 'success' | 'error'; message: string } | null;

// ---- Icons as inline SVGs ----
const FolderIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
  </svg>
);

const CheckIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M20 6 9 17l-5-5" />
  </svg>
);

const AlertIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="10" />
    <line x1="12" x2="12" y1="8" y2="12" />
    <line x1="12" x2="12.01" y1="16" y2="16" />
  </svg>
);

const ImageIcon = () => (
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
    <circle cx="8.5" cy="8.5" r="1.5" />
    <polyline points="21 15 16 10 5 21" />
  </svg>
);

const Spinner = () => <span className="spinner" />;

// ---- Helpers ----
function formatDate(dateStr: string): string {
  return dateStr.replace(/-/g, '/');
}

function sanitizePreview(text: string): string {
  return text
    .replace(/[\\/:*?"<>|]/g, '_')
    .trim();
}

// ---- Main Component ----
function App() {
  const [folderPath, setFolderPath] = useState<string | null>(null);
  const [_entries, _setEntries] = useState<ImageEntry[]>([]);
  const [inferResult, setInferResult] = useState<InferResult | null>(null);
  const [location, setLocation] = useState('');
  const [title, setTitle] = useState('');
  const [loading, setLoading] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [feedback, setFeedback] = useState<Feedback>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
  const [history, setHistory] = useState<RenameEntry[]>([]);

  const [isDark, setIsDark] = useState(() => localStorage.getItem("darkMode") === "true");
  const [queue, _setQueue] = useState<QueueItem[]>([]);
  const [currentIndex, _setCurrentIndex] = useState(-1);

    const processFolder = useCallback(async (selected: string) => {
    if (!selected) return;
    setFeedback(null);
    setFolderPath(selected);
    setLocation('');
    setTitle('');
    setInferResult(null);
    setLoading(true);
    try {
      const scanResult: ImageEntry[] = await invoke('scan_folder', { folderPath: selected });
      if (scanResult.length === 0) {
        setFeedback({ type: 'error', message: '所选文件夹中没有找到图片文件' });
        _setEntries([]);
        setLoading(false);
        return;
      }
      _setEntries(scanResult);
      const inference: InferResult = await invoke('infer_date', { folderPath: selected, entries: scanResult });
      setInferResult(inference);
      // GPS geocode
      try {
        const gps = await invoke<[number, number] | null>("extract_gps", { folderPath: selected });
        if (gps) {
          const url_ = "https://nominatim.openstreetmap.org/reverse?lat=" + gps[0] + "&lon=" + gps[1] + "&format=json&accept-language=zh";
          const resp = await fetch(url_, { headers: { "User-Agent": "image-archive/0.1" } });
          const data = await resp.json();
          const city = data.address?.city || data.address?.town || data.address?.village || data.address?.county;
          if (city) setLocation(city);
        }
      } catch (_e) {}
    } catch (err) {
      setFeedback({ type: 'error', message: '扫描失败: ' + err });
    } finally {
      setLoading(false);
    }
  }, []);



  const handleSelectFolder = useCallback(async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected) return;

      setFeedback(null);
      setFolderPath(selected);
      setLocation('');
      setTitle('');
      setInferResult(null);
      setLoading(true);

      const scanResult: ImageEntry[] = await invoke('scan_folder', {
        folderPath: selected,
      });

      if (scanResult.length === 0) {
        setFeedback({ type: 'error', message: '所选文件夹中没有找到图片文件' });
        _setEntries([]);
        setLoading(false);
        return;
      }

      _setEntries(scanResult);
      const inference: InferResult = await invoke('infer_date', {
        folderPath: selected,
        entries: scanResult,
      });
      setInferResult(inference);
      // GPS geocode
      try {
        const gps_data = await invoke<[number, number] | null>("extract_gps", { folderPath: selected });
        if (gps_data) {
          const url = "https://nominatim.openstreetmap.org/reverse?lat=" + gps_data[0] + "&lon=" + gps_data[1] + "&format=json&accept-language=zh";
          const resp = await fetch(url, { headers: { "User-Agent": "image-archive/0.1" } });
          const data = await resp.json();
          const city = data.address?.city || data.address?.town || data.address?.village || data.address?.county;
          if (city) setLocation(city);
        }
      } catch (_e) {}
    } catch (err) {
      setFeedback({ type: 'error', message: `扫描失败: ${err}` });
    } finally {
      setLoading(false);
    }
  }, []);

  const handleRename = useCallback(async () => {
    if (!folderPath || !inferResult?.date || !location.trim() || !title.trim()) return;

    setRenaming(true);
    setFeedback(null);

    try {
      const newPath: string = await invoke('rename_folder', {
        oldPath: folderPath,
        location: location.trim(),
        title: title.trim(),
        date: inferResult.date,
      });

      const shortName = newPath.split('\\').pop() || newPath;
      setFeedback({ type: 'success', message: `归档完成！\n${shortName}` });
    } catch (err) {
      setFeedback({ type: 'error', message: `重命名失败: ${err}` });
    } finally {
      setRenaming(false);
    }
  }, [folderPath, inferResult, location, title, queue, currentIndex]);

  const previewName = useMemo(() => {
    if (!inferResult?.date) return null;
    const loc = sanitizePreview(location);
    const t = sanitizePreview(title);
    if (!loc && !t) return inferResult.date;
    return `${inferResult.date}_${loc}_${t}`;
  }, [inferResult, location, title]);

  
  useEffect(() => {
    if (feedback?.type === 'success' && folderPath) {
      invoke<RenameEntry[]>('list_history', { folderPath }).then(setHistory).catch(() => {});
    }
  }, [feedback, folderPath]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    const setup = async () => {
      try {
        const win = getCurrentWindow();
        unlisten = await win.onDragDropEvent((event: { payload: { type: string; paths?: string[] } }) => {
          if (event.payload.type === 'over') {
            setIsDragOver(true);
          } else if (event.payload.type === 'leave') {
            setIsDragOver(false);
          } else if (event.payload.type === 'drop') {
            setIsDragOver(false);
            const paths = event.payload.paths;
            if (paths && paths.length > 0) {
              processFolder(paths[0]);
            }
          }
        });
      } catch (e) {}
    };
    setup();
    return () => { if (unlisten) unlisten(); };
  }, [processFolder]);

  const handleUndo = useCallback(async () => {
    if (!folderPath) return;
    try {
      const [oldName, newName] = await invoke<[string, string]>('undo_rename', { folderPath });
      setFeedback({ type: 'success', message: '已撤销: ' + newName + ' → ' + oldName });
      const h = await invoke<RenameEntry[]>('list_history', { folderPath });
      setHistory(h);
    } catch (err) {
      setFeedback({ type: 'error', message: '撤销失败: ' + err });
    }
  }, [folderPath]);

  const handleSelectDate = useCallback((date: string) => {
    if (!inferResult) return;
    setSelectedDate(date);
    setInferResult({ ...inferResult, date });
  }, [inferResult]);

  const canRename = inferResult?.date && location.trim() && title.trim() && !renaming;

  return (
    <div className={"app-container"+(isDark?" dark":"")}>
      {/* Header */}
      <header className="app-header">
        <h1>图片归档工具</h1>
        <p>按日期和城市重命名文件夹</p><button className="dark-toggle" onClick={() => { const d = !isDark; setIsDark(d); localStorage.setItem("darkMode", String(d)); }}>{isDark ? "☀️" : "🌙"}</button>
      </header>

      {/* Folder Picker */}
      <div className="card">
        <div className="folder-picker">
          <button className="btn-primary" onClick={handleSelectFolder} disabled={loading}>
            {loading ? <Spinner /> : <FolderIcon />}
            {loading ? '扫描中...' : '选择文件夹'}
          </button>
          <p className="drag-hint">或拖拽文件夹到此处</p>
          {folderPath && (
            <div className="folder-path">{folderPath}</div>
          )}
        </div>
      </div>

      {/* Queue Panel */}
      {queue.length > 0 && (
        <div className="card" style={{ padding: '8px 12px' }}>
          <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 6 }}>
            第 {currentIndex + 1}/{queue.length} 个文件夹
          </div>
          <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
            {queue.map((item, i) => (
              <span
                key={i}
                style={{
                  padding: '2px 8px',
                  borderRadius: 'var(--radius-pill)',
                  fontSize: 11,
                  background: i === currentIndex ? 'var(--primary)' : 'var(--surface-card)',
                  color: i === currentIndex ? 'var(--on-primary)' : 'var(--muted)',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  maxWidth: 120,
                  whiteSpace: 'nowrap',
                }}
              >
                {item.folderName}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Drop Zone */}
      {isDragOver && (
        <div className="drop-zone-overlay">
          <span className="drop-zone-icon">📁</span>
          <span className="drop-zone-text">释放以选择文件夹</span>
        </div>
      )}



      {/* Scan Results */}
      {inferResult && inferResult.total > 0 && (
        <div className="card">
          <div className="card-title">
            <ImageIcon />
            {' '}共 {inferResult.total} 张图片
          </div>

          {/* Date Display */}
          {inferResult.date && (
            <div className="date-display">
              <span className="date-label">推断日期</span>
              <span className="date-value">{formatDate(inferResult.date)}</span>
              <span style={{ fontSize: 12, color: 'var(--muted)' }}>
                ({inferResult.count}/{inferResult.total} 张)
              </span>
            </div>
          )}

          {/* Date options when conflict */}
          {inferResult.has_conflict && inferResult.date_entries.length > 0 && (
            <div style={{ marginTop: '8px' }}>
              <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 6 }}>检测到多个日期，请选择：</div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
                {Array.from(new Set(inferResult.date_entries.filter(d => d.date).map(d => d.date))).map(date => (
                  <button
                    key={date}
                    onClick={() => handleSelectDate(date!)}
                    style={{
                      padding: '4px 12px',
                      borderRadius: 'var(--radius-pill)',
                      border: '1px solid var(--hairline)',
                      background: (inferResult.date === date || selectedDate === date) ? 'var(--primary)' : 'var(--canvas)',
                      color: (inferResult.date === date || selectedDate === date) ? 'var(--on-primary)' : 'var(--body)',
                      cursor: 'pointer',
                      fontSize: 13,
                      fontFamily: 'var(--font-mono)',
                    }}
                  >
                    {date!.replace(/_/g, '/')}
                  </button>
                ))}
              </div>
            </div>
          )}

          {!inferResult.date && (
            <div style={{ color: 'var(--warning)', fontSize: 13, padding: '8px 0' }}>
              未能从图片中提取到拍摄日期
            </div>
          )}

          {/* Conflict Warning */}
          {inferResult.has_conflict && inferResult.conflict_message && (
            <div className="conflict-warning">
              <AlertIcon />
              <span>{inferResult.conflict_message}</span>
            </div>
          )}

          {/* File List (collapsible) */}
          <details style={{ marginTop: 'var(--space-sm)' }}>
            <summary style={{ fontSize: 12, color: 'var(--muted)', cursor: 'pointer' }}>
              查看图片列表 ({inferResult.total})
            </summary>
            <div className="scan-result" style={{ marginTop: 8 }}>
              {inferResult.date_entries.map((de, i) => (
                <div className="scan-row" key={i} onClick={() => invoke("open_file", { path: de.path })} style={{ cursor: "pointer" }} title="点击打开图片">
                  <span className="filename"><img src={convertFileSrc(de.path)} style={{ width: 40, height: 40, objectFit: 'cover', borderRadius: 4, marginRight: 6, verticalAlign: 'middle' }} />{de.filename}</span>
                  {de.date ? (
                    <span className={`date${de.is_outlier ? ' outlier' : ''}`}>
                      {formatDate(de.date)}{de.date_source?<span className="date-source-badge">{de.date_source.replace(/^exif_/,"EXIF").replace("exif_dt","EXIF").replace("mtime","修改时间").replace("filename","文件名")}</span>:null}
                    </span>
                  ) : (
                    <span className="no-date">无日期</span>
                  )}
                </div>
              ))}
            </div>
          </details>
        </div>
      )}

      {/* Empty State */}
      {!loading && !inferResult && (
        <div className="card">
          <div className="empty-state">
            <ImageIcon />
            <p>选择一个文件夹开始</p>
          </div>
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div className="card">
          <div className="empty-state">
            <Spinner />
            <p style={{ marginTop: 8 }}>正在扫描图片并提取日期...</p>
          </div>
        </div>
      )}

      {/* Input Form */}
      {inferResult && inferResult.date && (
        <div className="card">
          <div className="input-row">
            <div className="input-group">
              <label htmlFor="location">城市</label>
              <input
                id="location"
                className="input-field"
                placeholder="例如：北京"
                value={location}
                onChange={(e) => setLocation(e.target.value)}
                disabled={renaming}
              />
            </div>
            <div className="input-group">
              <label htmlFor="title">标题</label>
              <input
                id="title"
                className="input-field"
                placeholder="例如：踏春赏花"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                disabled={renaming}
              />
            </div>
          </div>
        </div>
      )}

      {/* Preview */}
      {previewName && (
        <div className="preview-box">
          <div className="preview-label">文件夹将重命名为</div>
          <div className="preview-name">{previewName}</div>
          <div className="preview-hint">确认后执行归档，此操作不可撤销</div>
        </div>
      )}

      {/* Feedback */}
      {feedback && (
        <div className={`feedback ${feedback.type}`}>
          {feedback.type === 'success' ? <CheckIcon /> : <AlertIcon />}
          <span>{feedback.message}</span>
        </div>
      )}

            {/* Action Button */}
      <button
        className="btn-primary btn-full"
        disabled={!canRename}
        onClick={handleRename}
      >
        {renaming ? <Spinner /> : <FolderIcon />}
        {renaming ? '归档中...' : '开始归档'}
      </button>

      {/* Footer */}
      {history.length > 0 && (
        <div className="card">
          <div className="card-title">操作历史</div>
          {history.slice().reverse().slice(0, 5).map((entry, i) => (
            <div key={i} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '6px 0', borderBottom: '1px solid var(--hairline-soft)', fontSize: 13 }}>
              <span style={{ flex: 1, minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {entry.old_name} → {entry.new_name}
              </span>
              <button
                onClick={() => handleUndo()}
                style={{ fontSize: 12, color: 'var(--primary)', background: 'none', border: 'none', cursor: 'pointer', flexShrink: 0, marginLeft: 8 }}
              >
                撤销
              </button>
            </div>
          ))}
        </div>
      )}

      <footer style={{
        textAlign: 'center',
        fontSize: 12,
        color: 'var(--muted-soft)',
        padding: 'var(--space-md) 0',
        marginTop: 'auto',
      }}>
        图片归档工具 v0.1
      </footer>
    </div>
  );
}

export default App;
