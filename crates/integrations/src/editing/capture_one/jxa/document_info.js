function run(argv) {
  const a = Application("Capture One");
  if (!a.running()) throw Error("Capture One is not running");
  const d = a.documents.byId(argv[0]);
  if (!d.exists()) throw Error("document not open");
  const root = d.collections.whose({id: "capture"})();
  const camera = d.camera();
  return JSON.stringify({kind: d.kind(), captures: d.captures().toString(), capture_root: root[0].file().toString(), camera: camera ? camera.name() : null});
}
