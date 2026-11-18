function canonicalFolder(path) {
  return path.replace(/^\/private(?=\/)/, '').replace(/\/$/, '');
}
// A20 (owner 2026-11-18): the folders of all collections in one Apple event (`folder of every
// collection`; non-folder collections answer null) instead of one event per collection, so finding a
// session's collection costs the same with 10 or 1,000 sessions (dev Mac, 16 collections: 10 ms
// against 434 ms). Asks one by one only if Capture One refuses the bulk read.
function collectionForFolder(doc, folder) {
  const target = canonicalFolder(folder);
  let folders = null;
  try { folders = doc.collections.folder(); } catch (_) { folders = null; }
  if (folders !== null) {
    for (let i = 0; i < folders.length; i++) {
      const value = folders[i];
      if (value !== null && value !== undefined && canonicalFolder(value.toString()) === target) return doc.collections.at(i);
    }
    return null;
  }
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
