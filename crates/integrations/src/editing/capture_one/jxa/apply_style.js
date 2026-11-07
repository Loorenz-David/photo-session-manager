function run(argv) {
  const app = Application('Capture One');
  if (!app.running()) throw Error('Capture One is not running');
  if (!app.availableStyles().includes(argv[4])) return JSON.stringify(false);
  const image = imageByPath(app.documents.byId(argv[0]), argv[1], argv[2]);
  if (image === null) throw Error('variant not found');
  const variants = image.variants.whose({id: argv[3]})();
  if (!variants.length) throw Error('variant not found');
  if (variants[0].styles().includes(argv[4])) return JSON.stringify(true);
  const layer = variants[0].layers()[0];
  if (!layer) throw Error('image layer not found');
  app.applyStyle(layer, {named: argv[4]});
  if (!variants[0].styles().includes(argv[4])) throw Error('style not applied');
  return JSON.stringify(true);
}
