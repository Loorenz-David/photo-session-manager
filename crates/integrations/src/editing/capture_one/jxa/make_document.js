function run(argv) {
  const a = Application('Capture One');
  const d = a.make({new:'document', withProperties:{name:argv[1], path:Path(argv[0])}});
  return JSON.stringify(d.id());
}
