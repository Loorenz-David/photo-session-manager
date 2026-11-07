function run(argv) {
  const app = Application('Capture One');
  if (!app.running()) throw Error('Capture One is not running');
  const doc = app.documents.byId(argv[0]);
  const want = canonicalFolder(argv[1]);
  const current = doc.currentCollection();
  if (current !== null && current.file() !== null && canonicalFolder(current.file().toString()) === want) return JSON.stringify(null);
  const collection = collectionForFolder(doc, argv[1]);
  if (collection === null) throw Error('current collection not found');
  doc.currentCollection = collection;
  const read = doc.currentCollection();
  if (read === null || read.file() === null || canonicalFolder(read.file().toString()) !== want) throw Error('current collection not set');
  return JSON.stringify(null);
}
