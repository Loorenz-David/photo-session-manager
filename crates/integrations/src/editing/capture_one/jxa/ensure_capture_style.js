function run(argv) {
  const a = Application('Capture One');
  const doc = a.documents.byId(argv[0]);
  const listed = a.availableStyles().includes(argv[1]);
  const options = doc.nextCaptureSettings;
  const current = options.applyStyles();
  if (listed && (current.length !== 1 || current[0] !== argv[1])) options.applyStyles = [argv[1]];
  return JSON.stringify({listed, applied:options.applyStyles()});
}
