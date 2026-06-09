import { useState, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
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
  return dateStr.replace(/_/g, '/');
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
        setFeedback({ type: 'error', message: '鎵€閫夋枃浠跺す涓病鏈夋壘鍒板浘鐗囨枃浠? });
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
    } catch (err) {
      setFeedback({ type: 'error', message: `鎵弿澶辫触: ${err}` });
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
      setFeedback({ type: 'success', message: `褰掓。瀹屾垚锛乗n${shortName}` });
    } catch (err) {
      setFeedback({ type: 'error', message: `閲嶅懡鍚嶅け璐? ${err}` });
    } finally {
      setRenaming(false);
    }
  }, [folderPath, inferResult, location, title]);

  const previewName = useMemo(() => {
    if (!inferResult?.date) return null;
    const loc = sanitizePreview(location);
    const t = sanitizePreview(title);
    if (!loc && !t) return inferResult.date;
    return `${inferResult.date}_${loc}_${t}`;
  }, [inferResult, location, title]);

  const canRename = inferResult?.date && location.trim() && title.trim() && !renaming;

  return (
    <div className="app-container">
      {/* Header */}
      <header className="app-header">
        <h1>鍥剧墖褰掓。宸ュ叿</h1>
        <p>鎸夋媿鎽勬棩鏈熷拰鍦扮偣鐨勭粺涓€閲嶅懡鍚嶆枃浠跺す</p>
      </header>

      {/* Folder Picker */}
      <div className="card">
        <div className="folder-picker">
          <button className="btn-primary" onClick={handleSelectFolder} disabled={loading}>
            {loading ? <Spinner /> : <FolderIcon />}
            {loading ? '鎵弿涓?..' : '閫夋嫨鏂囦欢澶?}
          </button>
          {folderPath && (
            <div className="folder-path">{folderPath}</div>
          )}
        </div>
      </div>

      {/* Feedback */}
      {feedback && (
        <div className={`feedback ${feedback.type}`}>
          {feedback.type === 'success' ? <CheckIcon /> : <AlertIcon />}
          <span>{feedback.message}</span>
        </div>
      )}

      {/* Scan Results */}
      {inferResult && inferResult.total > 0 && (
        <div className="card">
          <div className="card-title">
            <ImageIcon />
            {' '}鍏?{inferResult.total} 寮犲浘鐗?          </div>

          {/* Date Display */}
          {inferResult.date && (
            <div className="date-display">
              <span className="date-label">鎺ㄦ柇鏃ユ湡</span>
              <span className="date-value">{formatDate(inferResult.date)}</span>
              <span style={{ fontSize: 12, color: 'var(--muted)' }}>
                ({inferResult.count}/{inferResult.total} 寮?
              </span>
            </div>
          )}

          {!inferResult.date && (
            <div style={{ color: 'var(--warning)', fontSize: 13, padding: '8px 0' }}>
              鏈兘浠庡浘鐗囦腑鎻愬彇鍒版媿鎽勬棩鏈?            </div>
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
              鏌ョ湅鍥剧墖鍒楄〃 ({inferResult.total})
            </summary>
            <div className="scan-result" style={{ marginTop: 8 }}>
              {inferResult.date_entries.map((de, i) => (
                <div className="scan-row" key={i}>
                  <span className="filename">{de.filename}</span>
                  {de.date ? (
                    <span className={`date${de.is_outlier ? ' outlier' : ''}`}>
                      {formatDate(de.date)}
                    </span>
                  ) : (
                    <span className="no-date">鏃犳棩鏈?/span>
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
            <p>閫夋嫨涓€涓枃浠跺す寮€濮?/p>
          </div>
        </div>
      )}

      {/* Loading */}
      {loading && (
        <div className="card">
          <div className="empty-state">
            <Spinner />
            <p style={{ marginTop: 8 }}>姝ｅ湪鎵弿鍥剧墖骞舵彁鍙栨棩鏈?..</p>
          </div>
        </div>
      )}

      {/* Input Form */}
      {inferResult && inferResult.date && (
        <div className="card">
          <div className="input-row">
            <div className="input-group">
              <label htmlFor="location">鍦扮偣</label>
              <input
                id="location"
                className="input-field"
                placeholder="渚嬪锛氬寳浜鍜屽洯"
                value={location}
                onChange={(e) => setLocation(e.target.value)}
                disabled={renaming}
              />
            </div>
            <div className="input-group">
              <label htmlFor="title">鏍囬</label>
              <input
                id="title"
                className="input-field"
                placeholder="渚嬪锛氳笍鏄ヨ祻鑺?
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
          <div className="preview-label">鏂囦欢澶瑰皢閲嶅懡鍚嶄负</div>
          <div className="preview-name">{previewName}</div>
          <div className="preview-hint">纭鍚庢墽琛屽綊妗ｏ紝姝ゆ搷浣滀笉鍙挙閿€</div>
        </div>
      )}

      {/* Action Button */}
      <button
        className="btn-primary btn-full"
        disabled={!canRename}
        onClick={handleRename}
      >
        {renaming ? <Spinner /> : <FolderIcon />}
        {renaming ? '褰掓。涓?..' : '寮€濮嬪綊妗?}
      </button>

      {/* Footer */}
      <footer style={{
        textAlign: 'center',
        fontSize: 12,
        color: 'var(--muted-soft)',
        padding: 'var(--space-md) 0',
        marginTop: 'auto',
      }}>
        鍥剧墖褰掓。宸ュ叿 v0.1
      </footer>
    </div>
  );
}

export default App;
