function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const doc = a.documents.byId(argv[0]);
  const images = doc.images();
  for (const image of images) {
    const variants = image.variants();
    if (variants.length) return JSON.stringify(variants[0].availableCropAspectRatios());
  }
  for (const collection of doc.collections()) {
    for (const image of collection.images()) {
      const variants = image.variants();
      if (variants.length) return JSON.stringify(variants[0].availableCropAspectRatios());
    }
  }
  return JSON.stringify([]);
}
