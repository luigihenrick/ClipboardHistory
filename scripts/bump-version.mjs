#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const packagePath = join(root, "package.json");
const lockPath = join(root, "package-lock.json");
const tauriConfigPath = join(root, "src-tauri", "tauri.conf.json");

const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
const writeJson = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);

const packageJson = readJson(packagePath);
const current = packageJson.version;
const input = process.argv.slice(2).find((arg) => arg !== "--") ?? "patch";

if (input === "--help" || input === "-h") {
  console.log(`Usage: npm run version:bump -- <version|major|minor|patch>

Examples:
  npm run version:bump              # patch bump
  npm run version:bump -- minor
  npm run version:bump -- 1.3.0`);
  process.exit(0);
}

const parse = (value) => {
  const match = /^(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?$/.exec(value.replace(/^v/, ""));
  return match
    ? { major: Number(match[1]), minor: Number(match[2]), patch: Number(match[3]) }
    : null;
};

const parsed = parse(current);
if (!parsed) {
  throw new Error(`package.json has an invalid semver version: ${current}`);
}

let next;
if (input === "major") {
  next = `${parsed.major + 1}.0.0`;
} else if (input === "minor") {
  next = `${parsed.major}.${parsed.minor + 1}.0`;
} else if (input === "patch") {
  next = `${parsed.major}.${parsed.minor}.${parsed.patch + 1}`;
} else {
  if (!parse(input)) {
    throw new Error(`Invalid version or bump type: ${input}`);
  }
  next = input.replace(/^v/, "");
}

if (next === current) {
  throw new Error(`Version is already ${current}`);
}

const tauriConfig = readJson(tauriConfigPath);
if (tauriConfig.version !== "../package.json") {
  throw new Error(
    'src-tauri/tauri.conf.json must keep "version": "../package.json" so package.json is the single source of truth.',
  );
}

packageJson.version = next;
writeJson(packagePath, packageJson);

const packageLock = readJson(lockPath);
packageLock.version = next;
if (!packageLock.packages?.[""]) {
  throw new Error("package-lock.json is missing the root package entry");
}
packageLock.packages[""].version = next;
writeJson(lockPath, packageLock);

console.log(`ClipHistory version: ${current} -> ${next}`);
console.log("Updated package.json and package-lock.json.");
console.log("tauri.conf.json reads package.json directly; the Cargo crate keeps a static version.");
console.log("\nNext steps:");
console.log("  1. Update .github/release-notes.md");
console.log(`  2. git add -A && git commit -m "Release v${next}"`);
console.log(`  3. git tag v${next} && git push origin main --tags`);
