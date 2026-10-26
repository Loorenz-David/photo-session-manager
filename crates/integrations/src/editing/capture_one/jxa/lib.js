function canonicalFolder(path) {
  return path.replace(/^\/private(?=\/)/, '').replace(/\/$/, '');
}
function collectionForFolder(doc, folder) {
  const target = canonicalFolder(folder);
  for (const collection of doc.collections()) {
    const value = collection.folder();
    if (value !== null && canonicalFolder(value.toString()) === target) return collection;
  }
  return null;
}
function imageByPath(doc, raw, folder) {
  let images = doc.images.whose({path: raw})();
  if (images.length) return images[0];
  const collection = collectionForFolder(doc, folder);
  if (collection === null) return null;
  images = collection.images.whose({path: raw})();
  return images.length ? images[0] : null;
}
