function run(argv) { const a=Application("Capture One"); if(!a.running()) throw Error("Capture One is not running"); a.save(a.documents.byId(argv[0])); return JSON.stringify(null); }
