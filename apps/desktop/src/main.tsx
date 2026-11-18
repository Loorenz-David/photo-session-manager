import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { SearchWindow, searchRoute } from './features/search/SearchWindow';
import './styles/tokens.css';
import './styles/globals.css';

// A12: a "Search this photo" window loads the same bundle on `#/search/<session>/<photo>` and shows only its own page
// (none of the main window's work, e.g. pointing Capture One at a session, runs there).
const search = searchRoute(window.location.hash);
const label: string | undefined = (window as { __TAURI_INTERNALS__?: { metadata?: { currentWebview?: { label?: string } } } }).__TAURI_INTERNALS__?.metadata?.currentWebview?.label;
const inSearchWindow = label?.startsWith('search-ui-') ?? false;

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {search ? <SearchWindow sessionId={search.sessionId} initialPhotoId={search.photoId} /> : inSearchWindow ? <p className="p-4 text-sm">This search window could not open. Close it and tap the photo again.</p> : <App />}
  </React.StrictMode>,
);
