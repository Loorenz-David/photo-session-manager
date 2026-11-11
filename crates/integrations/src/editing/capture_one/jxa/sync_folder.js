// P65 (changelog 45): asks Capture One to synchronise a session folder collection with the files on
// disk (sdef §collection `synchronizing`: "can be set to true for a session folder collection in
// order to initiate background synchronization"), so a photo the app filed is listed in seconds.
function run(argv) {
  const app = Application('Capture One');
  if (!app.running()) throw Error('Capture One is not running');
  const doc = app.documents.byId(argv[0]);
  const collection = collectionForFolder(doc, argv[1]);
  if (collection === null) throw Error('collection not found');
  collection.synchronizing = true;
  return JSON.stringify(null);
}
