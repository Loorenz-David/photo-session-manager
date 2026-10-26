function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const image = imageByPath(a.documents.byId(argv[0]), argv[1], argv[2]);
  if (image === null) throw Error('variant not found');
  return JSON.stringify(image.variants.byId(argv[3]).colorTag());
}
