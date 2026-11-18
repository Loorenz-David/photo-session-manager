// A20 (owner 2026-11-18, "delays when opening a session"): a fixed handful of Apple events. It used
// to list every collection (three events each; one folder collection per session), ~3.5 s a call on
// the work Mac with ~125 collections and growing with every session, and nothing read the list.
function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const d = a.documents.byId(argv[0]);
  if (!d.exists()) throw Error('document not open');
  const camera = d.camera();
  return JSON.stringify({kind: d.kind(), id: d.id(), captures: d.captures().toString(), camera: camera ? camera.name() : null});
}
