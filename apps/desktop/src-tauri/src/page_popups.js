// Owner correction A14 (2026-11-18): links and pop-ups a page opens load in the same browser area.
//
// Evaluated at document start in every frame of the "Search this photo" window's page webview
// (search_window.rs wraps this expression as `(<this>)(<go>)`). It only changes WHERE a page's own
// navigation goes: a new window becomes this page. It reads nothing, sends nothing and calls
// nothing in the app; every navigation still passes the app's navigation check
// (`allowed_navigation`). Only http(s) addresses are followed.
//
// Why: WebKit opens a new window for a page only from a direct click. Google's results open many
// links from script (a click handler that logs first and then calls `window.open`, or an empty
// window whose address is set afterwards). Without a direct click WebKit drops those silently, so
// clicking them did nothing.
(function (go) {
  'use strict';

  /** The absolute http(s) address of `u` (relative to this page), or null. */
  function web(u) {
    try {
      var a = new URL(String(u), document.baseURI);
      return a.protocol === 'http:' || a.protocol === 'https:' ? a.href : null;
    } catch (e) {
      return null;
    }
  }

  function openHere(u) {
    var href = web(u);
    if (href) go(href);
    return !!href;
  }

  /** Targets that stay in this browsing context (not a new window). */
  function sameWindow(target) {
    var t = target == null ? '' : String(target).toLowerCase();
    return t === '' || t === '_self' || t === '_top' || t === '_parent';
  }

  /**
   * What `window.open` hands back: a stand-in window whose address, once set, opens here (pages
   * often open an empty window first and set its address afterwards).
   */
  function standIn() {
    var location = {
      assign: openHere,
      replace: openHere,
      reload: function () {},
      toString: function () {
        return 'about:blank';
      },
    };
    Object.defineProperty(location, 'href', {
      get: function () {
        return 'about:blank';
      },
      set: openHere,
    });
    var stand = {
      closed: false,
      opener: null,
      close: function () {
        stand.closed = true;
      },
      focus: function () {},
      blur: function () {},
      postMessage: function () {},
      document: { write: function () {}, writeln: function () {}, open: function () {}, close: function () {} },
    };
    Object.defineProperty(stand, 'location', {
      get: function () {
        return location;
      },
      set: openHere,
    });
    return stand;
  }

  var nativeOpen = window.open;
  window.open = function (url, target) {
    var t = target == null ? '' : String(target).toLowerCase();
    if (t === '_self' || t === '_top' || t === '_parent') return nativeOpen.apply(window, arguments);
    if (url != null && url !== '' && String(url) !== 'about:blank') openHere(url);
    return standIn();
  };

  // A link or form aimed at a new window is aimed at this page instead (before the page's own
  // handlers run, so a link the page lets through navigates here).
  function retarget(event) {
    var path = event.composedPath ? event.composedPath() : [event.target];
    for (var i = 0; i < path.length; i++) {
      var node = path[i];
      var tag = node && node.tagName ? String(node.tagName).toUpperCase() : '';
      if (tag === 'A' || tag === 'AREA' || tag === 'FORM') {
        if (!sameWindow(node.getAttribute('target'))) node.setAttribute('target', '_top');
        return;
      }
    }
  }
  window.addEventListener('click', retarget, true);
  window.addEventListener('submit', retarget, true);
})
