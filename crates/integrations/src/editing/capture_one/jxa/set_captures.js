function run(argv) { const a=Application("Capture One"); if(!a.running()) throw Error("Capture One is not running"); a.documents.byId(argv[0]).captures=Path(argv[1]); return JSON.stringify(null); }
