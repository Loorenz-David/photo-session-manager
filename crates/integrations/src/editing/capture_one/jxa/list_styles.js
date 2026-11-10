function run() {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  return JSON.stringify(a.availableStyles());
}
