import { SessionGrid } from './components/SessionGrid';
import { useSessions } from './hooks/useSessions';

function App() {
  const {
    sessions,
    totalCount,
    waitingCount,
    isLoading,
    error,
    refresh,
    focusSession,
  } = useSessions();

  return (
    <div className="min-h-screen bg-[#0d0d0d] flex flex-col">
      {/* Draggable title bar area */}
      <div
        data-tauri-drag-region
        className="h-12 flex items-center justify-between px-6 border-b border-white/5 bg-[#0d0d0d]"
      >
        <div className="flex items-center gap-3 pl-16">
          <h1 className="text-lg font-semibold text-white/90">Claude Sessions</h1>
          {totalCount > 0 && (
            <span className="text-sm text-white/40">
              {totalCount} active{waitingCount > 0 && ` · ${waitingCount} waiting`}
            </span>
          )}
        </div>
        <button
          onClick={refresh}
          disabled={isLoading}
          className="p-2 rounded-lg hover:bg-white/5 transition-colors text-white/50 hover:text-white/80 disabled:opacity-50"
          title="Refresh"
        >
          <svg
            className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
        </button>
      </div>

      {/* Main content area */}
      <div className="flex-1 overflow-y-auto p-6">
        {error ? (
          <div className="flex items-center justify-center h-full">
            <div className="p-6 text-red-400 text-sm text-center bg-red-500/10 rounded-xl border border-red-500/20">
              {error}
            </div>
          </div>
        ) : sessions.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <div className="w-16 h-16 mb-4 rounded-2xl bg-white/5 flex items-center justify-center">
              <svg className="w-8 h-8 text-white/20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
            </div>
            <p className="text-white/40 text-sm">No active Claude sessions</p>
            <p className="text-white/20 text-xs mt-1">Start a Claude session in your terminal to see it here</p>
          </div>
        ) : (
          <SessionGrid
            sessions={sessions}
            onSessionClick={focusSession}
          />
        )}
      </div>
    </div>
  );
}

export default App;
