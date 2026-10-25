// about.js — the About dialog: what is running, which build, under which
// licenses, and where its source lives.
//
// Two entries, the package's first when there is one:
//
//   ┌ About ─────────────────────────────────────────────┐
//   │ CoSy 2026.10                (the package's "about") │
//   │ credits · links                                     │
//   │ ─────────────────────────────────────────────────── │
//   │ on reva-fork 2026.1025.0 · build 64cc7aa 2026-10-25 │
//   │ credits · links · licenses                          │
//   └─────────────────────────────────────────────────────┘
//
// The package's entry comes from the "about" object of reva-package.json
// (the browser's ?package= loader and the desktop app both pass it on):
//
//   "about": { "name": "CoSy", "version": "...", "description": "...",
//              "credits": ["CoSy: Bob Armstrong", ...],
//              "links": [{ "label": "CoSy.com", "url": "https://..." }] }
//
// Build information is the `reva-build` meta tag ("VERSION COMMIT DATE"),
// stamped by make-web-zip.sh and desktop/build.sh; a page served from the
// source tree says "development build".  Links must be http(s); anything
// else is shown as text.  All text goes in with textContent, never HTML,
// so a manifest cannot inject markup.  In the desktop app, links open in
// the system browser (host.openUrl).

const REPO = "https://github.com/softwarewrighter/reva-fork";

/** reva-fork's own entry. */
export const REVA = {
  name: "reva-fork",
  description: "Reva Forth, ported to a portable 64-bit core and WebAssembly.",
  credits: [
    "Reva Forth: Ron Aaron and contributors (public domain)",
    "Portable core, WebAssembly, desktop app: Michael A. Wright (MIT)",
  ],
  links: [
    { label: "Source (GitHub)", url: REPO },
    { label: "Documentation", url: `${REPO}/tree/main/docs` },
  ],
  licenses: [
    { label: "LICENSE (Reva, public domain)", url: `${REPO}/blob/main/LICENSE` },
    { label: "LICENSE-MIT (portable core)", url: `${REPO}/blob/main/LICENSE-MIT` },
    { label: "Third-party licenses", url: `${REPO}/blob/main/docs/licenses.md` },
  ],
};

/** The `reva-build` meta tag as `{ version, commit, date }`, or null. */
export function buildInfo(doc = globalThis.document) {
  const s = doc?.querySelector('meta[name="reva-build"]')?.content?.trim();
  if (!s) return null;
  const [version, commit, date] = s.split(/\s+/);
  return { version, commit, date };
}

/** Only http(s) URLs become links. */
export const safeUrl = (u) => {
  try {
    const url = new URL(u, globalThis.location?.href);
    return url.protocol === "https:" || url.protocol === "http:" ? url.href : null;
  } catch {
    return null;
  }
};

/** A package's "about", checked: strings and lists of the right shape only. */
export function packageAbout(m) {
  const a = m?.about && typeof m.about === "object" ? m.about : {};
  const str = (v) => (typeof v === "string" && v.length < 500 ? v : undefined);
  const name = str(a.name) ?? str(m?.title) ?? str(m?.name);
  if (!name) return null;
  return {
    name,
    version: str(a.version),
    description: str(a.description),
    credits: Array.isArray(a.credits) ? a.credits.map(str).filter(Boolean).slice(0, 20) : [],
    links: Array.isArray(a.links)
      ? a.links.filter((l) => str(l?.label) && str(l?.url)).slice(0, 20)
      : [],
  };
}

const el = (tag, props = {}, ...kids) => {
  const e = document.createElement(tag);
  Object.assign(e, props);
  e.append(...kids);
  return e;
};

function linkList(links, openUrl) {
  const ul = el("ul");
  for (const { label, url } of links) {
    const href = safeUrl(url);
    const li = el("li");
    if (href) {
      const a = el("a", { href, textContent: label, target: "_blank", rel: "noopener noreferrer" });
      if (openUrl) a.addEventListener("click", (e) => { e.preventDefault(); openUrl(href); });
      li.append(a);
    } else {
      li.textContent = `${label}: ${url}`;
    }
    ul.append(li);
  }
  return ul;
}

function entry({ name, version, description, credits = [], links = [], licenses = [] }, openUrl, heading) {
  const s = el("section");
  s.append(el("h2", { textContent: heading ?? (version ? `${name} ${version}` : name) }));
  if (description) s.append(el("p", { textContent: description }));
  if (credits.length) s.append(el("ul", { className: "credits" }, ...credits.map((c) => el("li", { textContent: c }))));
  if (links.length) s.append(linkList(links, openUrl));
  if (licenses.length) s.append(el("p", { className: "muted", textContent: "Licenses:" }), linkList(licenses, openUrl));
  return s;
}

/**
 * Make the About dialog (hidden until `open()`).  `pkg`: a package's
 * about (see packageAbout) or null; `openUrl(url)`: the host's way to open
 * a link outside the page, if it has one.  Returns `{ open() }`.
 */
export function aboutDialog({ pkg = null, openUrl = null } = {}) {
  const build = buildInfo();
  const stamp = build
    ? `build ${[build.commit, build.date].filter(Boolean).join(" · ")}`
    : "development build (source tree)";
  const reva = { ...REVA, version: build?.version };
  const close = el("button", { type: "button", textContent: "Close", className: "close" });
  const d = el("dialog", { id: "about" });
  d.setAttribute("aria-label", `About ${pkg?.name ?? "Reva Forth"}`);
  if (pkg) d.append(entry(pkg, openUrl));
  const revaHead = `${pkg ? "on " : ""}reva-fork${reva.version ? ` ${reva.version}` : ""}`;
  d.append(entry(reva, openUrl, revaHead), el("p", { className: "muted", textContent: stamp }), close);
  close.addEventListener("click", () => d.close());
  // a click on the backdrop (the dialog element itself, outside its content) closes it
  d.addEventListener("click", (e) => {
    if (e.target !== d) return;
    const r = d.getBoundingClientRect();
    const inside = e.clientX >= r.left && e.clientX <= r.right && e.clientY >= r.top && e.clientY <= r.bottom;
    if (!inside) d.close();
  });
  document.body.append(d);
  return {
    open() {
      if (typeof d.showModal === "function") d.showModal();
      else d.setAttribute("open", "");
      close.focus();
    },
  };
}
