reva-web-2026.1026.0: Reva Forth in the browser (WebAssembly), as a static site.

Serve this folder with any static web server and open index.html:

  - on a web host: upload the zip, unzip it anywhere (a subfolder is fine),
    and open https://your.host/path/reva-web-2026.1026.0/
  - locally: python3 -m http.server -d reva-web-2026.1026.0 8000, then http://localhost:8000/
    (or run the reva-serve binary, which embeds the same files)

Opening index.html as a file:// URL does not work: browsers do not run
Web Workers or fetch files from file:// pages.

Files you write in Forth are kept in your browser's localStorage for this
site's origin (scheme, host and port): another host or port starts empty.

URL options: ?canvas=split|pane|window, ?package=DIR/ (another package
folder on the same site), ?include=FILE, ?eval=TEXT, ?image=FILE.

Licenses: LICENSE.txt (Reva, public domain), LICENSE-MIT.txt (the portable
core).
