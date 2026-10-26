function canonical(path) {
  return path.replace(/^\/private(?=\/)/, '').replace(/\/$/, '');
}

function run(argv) {
  const app = Application('Capture One');
  if (!app.running()) throw Error('Capture One is not running');
  const document = app.documents.byId(argv[0]);
  const folder = canonical(argv[1]);
  for (const collection of document.collections()) {
    const file = collection.file();
    if (file === null || canonical(file.toString()) !== folder) continue;
    const images = collection.images.whose({ path: argv[2] })();
    if (!images.length) return JSON.stringify(null);
    const pick = images[0].variants.whose({ position: 1 })();
    return JSON.stringify(pick.length ? pick[0].id().toString() : null);
  }
  return JSON.stringify(null);
}
