function run(argv) {
  const a = Application('Capture One');
  const doc = a.documents.byId(argv[0]);
  const want = JSON.parse(argv[2]);
  let r = doc.recipes.byName(argv[1]);
  const created = !r.exists();
  if (created) {
    const before = doc.recipes.name();
    try { r = a.make({new:'recipe', at:doc, withProperties:{name:argv[1]}}); }
    catch (e) {
      for (const candidate of doc.recipes()) {
        const name = candidate.name();
        if (name.startsWith('Untitled Recipe') && !before.includes(name)) a.delete(candidate);
      }
      throw e;
    }
  }
  const keys = ['enabled','outputFormat','colorProfile','scalingMethod','scalingUnit',
    'primaryScalingValue','jpegQuality','existingFiles','outputSubFolder'];
  const changed = [];
  for (const key of keys) {
    if (r[key]() !== want[key]) { r[key] = want[key]; changed.push(key); }
  }
  const props = {};
  for (const key of keys) props[key] = r[key]();
  props.rootFolderType = r.rootFolderType();
  return JSON.stringify({created, changed, props});
}
