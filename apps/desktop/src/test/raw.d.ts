// A file's text imported with Vite's `?raw` (A14: the page webview's script, tested in a14_page_popups).
declare module '*?raw' {
  const text: string;
  export default text;
}
