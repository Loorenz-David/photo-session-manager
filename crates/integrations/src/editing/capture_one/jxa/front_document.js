function run(argv) {
  const a = Application('Capture One');
  a.currentDocument = a.documents.byId(argv[0]);
  return JSON.stringify(a.currentDocument().id());
}
