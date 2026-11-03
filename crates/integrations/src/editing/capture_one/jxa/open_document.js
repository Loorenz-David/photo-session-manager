function run(argv) {
  const a = Application('Capture One');
  try { a.open(Path(argv[0])); }
  catch (error) {
    const front = a.currentDocument();
    if (front.name().startsWith('PSM Open Window')) a.close(front);
    throw error;
  }
  return JSON.stringify(null);
}
