import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
// master_plan_live §6: IBM Plex Mono ships with the phone build (never fetched from a font service).
import '@fontsource/ibm-plex-mono/latin-400.css';
import '@fontsource/ibm-plex-mono/latin-500.css';
import './phone.css';
import { PhoneApp } from './PhoneApp';

createRoot(document.getElementById('root')!).render(<StrictMode><PhoneApp /></StrictMode>);
