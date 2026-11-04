function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const image = imageByPath(a.documents.byId(argv[0]), argv[1], argv[2]);
  if (image === null) throw Error('variant not found');
  const variant = image.variants.byId(argv[3]);
  const adjustments = variant.adjustments;
  const geometry = {
    rotation: adjustments.rotation(),
    flip: String(adjustments.flip()),
    keystoneVertical: adjustments.keystoneVertical(),
    keystoneHorizontal: adjustments.keystoneHorizontal(),
    keystoneAmount: adjustments.keystoneAmount()
  };
  return JSON.stringify({job: a.process(variant, {recipe: argv[4]}).toString(), geometry});
}
