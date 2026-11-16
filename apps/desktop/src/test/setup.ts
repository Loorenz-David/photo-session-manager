// jsdom lacks the pointer-capture and scroll calls Radix dropdowns make; harmless no-ops for every test.
if (typeof Element !== 'undefined') {
  Element.prototype.scrollIntoView ??= function scrollIntoView() {};
  Element.prototype.hasPointerCapture ??= () => false;
  Element.prototype.releasePointerCapture ??= () => undefined;
}
