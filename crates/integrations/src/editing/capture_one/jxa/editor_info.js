function run() {
  const a = Application('Capture One');
  if (!a.running()) return JSON.stringify({running:false, version:null, pro:null});
  let pro = true;
  try { a.captureDoneScript(); } catch (_) { pro = false; }
  return JSON.stringify({running:true, version:a.version(), pro});
}
