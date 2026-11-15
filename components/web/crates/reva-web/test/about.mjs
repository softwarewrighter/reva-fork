// Node test for www/about.js: a package's manifest in the footer and About
// (CoSy's asks 23 and 25).  Run: node components/web/crates/reva-web/test/about.mjs
import { footerParts, packageAbout } from "../www/about.js";

let failed = 0;
const check = (what, got, want) => {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${JSON.stringify(got)}`);
};

const b = {
  name: "reva-fork", version: "2026.1106.2", copyright: "(c) 2026 Michael A. Wright",
  repository: "https://github.com/softwarewrighter/reva-fork",
  build_host: "mac", build_commit: "abc1234", build_time: "2026-11-06T10:20:30Z",
};
const tail = "reva-fork 2026.1106.2 · build abc1234 on mac, 2026-11-06 10:20";
const head = "© 2026 Michael A. Wright · MIT License · ";
const link = { link: b.repository };

check("no package: today's footer", footerParts(b), [head, link, " · ", tail]);
// CoSy's build/reva-package.json, as the page and the desktop app pass it
const cosy = { name: "cosy", title: "CoSy", about: { name: "CoSy", version: "2026.1106.2" } };
check("about.version leads", footerParts(b, packageAbout(cosy)), [head, link, " · ", `CoSy 2026.1106.2 on ${tail}`]);
check("name defaults to title", footerParts(b, packageAbout({ title: "T", about: { version: "1" } }))[3], `T 1 on ${tail}`);
check("no about.version: today's footer", footerParts(b, packageAbout({ title: "CoSy" })), [head, link, " · ", tail]);
const quiet = packageAbout({ ...cosy, sourceLinks: false });
check("sourceLinks: false drops the link", footerParts(b, quiet), [head, `CoSy 2026.1106.2 on ${tail}`]);
check("sourceLinks read", [packageAbout(cosy).sourceLinks, quiet.sourceLinks], [true, false]);
process.exit(failed ? 1 : 0);
