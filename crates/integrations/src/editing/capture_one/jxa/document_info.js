function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const d = a.documents.byId(argv[0]);
  if (!d.exists()) throw Error('document not open');
  const camera = d.camera();
  const collections = d.collections().map(c => {
    const folder = c.folder();
    return {id: c.id().toString(), name: c.name(), folder: folder === null ? null : folder.toString()};
  });
  return JSON.stringify({kind: d.kind(), id: d.id(), captures: d.captures().toString(), collections, camera: camera ? camera.name() : null});
}
