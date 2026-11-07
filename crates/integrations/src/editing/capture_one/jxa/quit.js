function run() { const a=Application("Capture One"); if(!a.running()) return JSON.stringify(null); a.quit(); return JSON.stringify(null); }
