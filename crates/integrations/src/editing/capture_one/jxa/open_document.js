function run(argv) {
  const a = Application('Capture One');
  try {
    a.make({new:'document', withProperties:{name:'PSM Open Window', path:Path(argv[1])}});
    a.open(Path(argv[0]));
  }
  catch (error) {
    const front = a.currentDocument();
    if (front && front.name().startsWith('PSM Open Window')) a.close(front);
    throw error;
  }
  return JSON.stringify(null);
}
