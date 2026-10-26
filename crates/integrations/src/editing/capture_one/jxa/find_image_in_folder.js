function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const collection = collectionForFolder(a.documents.byId(argv[0]), argv[1]);
  if (collection === null) return JSON.stringify(null);
  const images = collection.images.whose({path: argv[2]})();
  if (!images.length) return JSON.stringify(null);
  const pick = images[0].variants.whose({position: 1})();
  return JSON.stringify(pick.length ? pick[0].id().toString() : null);
}
