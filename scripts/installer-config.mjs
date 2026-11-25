// installer-config.mjs VERSION OUT [STAGE] — the tauri build config for an
// installer (scripts/make-installer.sh), written to OUT/tauri.json.  Prints
// "NAME<TAB>PACKAGE_VERSION<TAB>ICON" (ICON: the package's icon file, or "").
import { readFileSync, writeFileSync } from "node:fs";

const [version, out, stage] = process.argv.slice(2);
// semver for the bundlers: 2026.0105.0 -> 2026.105.0
const semver = (v) => (/^\d+(\.\d+){0,2}$/.test(v ?? "") ? v.split(".").map(Number).join(".") : null);
const config = { version: semver(version), bundle: { macOS: {} } };
// ad-hoc signed (free; Apple Silicon runs it) unless a real identity is given
if (!process.env.APPLE_SIGNING_IDENTITY) config.bundle.macOS.signingIdentity = "-";
let line = ["Reva", "", ""];
if (stage) {
  const m = JSON.parse(readFileSync(`${stage}/reva-package.json`, "utf8"));
  if (typeof m.name !== "string" || !m.name.trim()) throw new Error("reva-package.json needs a name");
  const slug = m.name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "app";
  const pkgVersion = typeof m.about?.version === "string" ? m.about.version : "";
  Object.assign(config, {
    productName: m.name,
    mainBinaryName: slug,
    identifier: typeof m.identifier === "string" ? m.identifier : `com.softwarewrighter.reva.${slug}`,
    version: semver(pkgVersion) ?? config.version,
  });
  config.bundle.resources = { [`${stage}/`]: "package/" };
  line = [m.name, pkgVersion, typeof m.icon === "string" ? m.icon : ""];
}
writeFileSync(`${out}/tauri.json`, JSON.stringify(config, null, 2) + "\n");
console.log(line.join("\t"));
