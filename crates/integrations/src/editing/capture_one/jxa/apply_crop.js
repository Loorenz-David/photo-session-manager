function run(argv) {
  const a = Application('Capture One');
  if (!a.running()) throw Error('Capture One is not running');
  const image = imageByPath(a.documents.byId(argv[0]), argv[1], argv[2]);
  if (image === null) throw Error('variant not found');
  const variant = image.variants.byId(argv[3]);
  if (!variant) throw Error('variant not found');
  const ratio = argv[4];
  const available = variant.availableCropAspectRatios();
  const current = variant.cropAspectRatio();
  const size = () => ({width: variant.cropWidth(), height: variant.cropHeight()});
  if (!available.includes(ratio)) return JSON.stringify({outcome: 'unknown_ratio', ratio: current, ...size(), available});
  if (current !== null && current !== undefined && current !== '' && current !== 'Unconstrained') {
    return JSON.stringify({outcome: 'skipped', ratio: current, ...size()});
  }
  const max = a.maximumCrop(variant, {apply: false});
  const width = max && max.x + max.width;
  const height = max && max.y + max.height;
  if (typeof width !== 'number' || typeof height !== 'number' || !Number.isFinite(width) || !Number.isFinite(height)) {
    throw Error('maximum crop size unavailable');
  }
  if (variant.cropWidth() !== width || variant.cropHeight() !== height) {
    return JSON.stringify({outcome: 'skipped', ratio: current, ...size()});
  }
  variant.cropAspectRatio = ratio;
  return JSON.stringify({outcome: 'applied', ratio: variant.cropAspectRatio(), ...size()});
}
