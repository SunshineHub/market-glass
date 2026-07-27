import { readFile, writeFile } from "node:fs/promises";
import process from "node:process";
import { pathToFileURL } from "node:url";

const PLATFORM_ARTIFACTS = {
  "darwin-aarch64": /_aarch64\.app\.tar\.gz$/,
  "darwin-aarch64-app": /_aarch64\.app\.tar\.gz$/,
  "windows-x86_64": /_x64-setup\.exe$/,
  "windows-x86_64-nsis": /_x64-setup\.exe$/,
};

function releaseAssetUrl(repository, tag, assetName) {
  return `https://github.com/${repository}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(assetName)}`;
}

function readAssetNames(payload) {
  const assets = Array.isArray(payload) ? payload : payload.assets;
  if (!Array.isArray(assets)) throw new Error("Release asset metadata must contain an assets array");
  return assets.map((asset) => asset?.name).filter((name) => typeof name === "string");
}

export async function rewriteUpdaterManifest({ manifestPath, assetsPath, repository, tag }) {
  if (!/^[^/]+\/[^/]+$/.test(repository)) throw new Error(`Invalid repository: ${repository}`);
  if (!/^v[^/]+$/.test(tag)) throw new Error(`Invalid release tag: ${tag}`);

  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const assetNames = readAssetNames(JSON.parse(await readFile(assetsPath, "utf8")));
  if (!manifest.platforms || typeof manifest.platforms !== "object") {
    throw new Error("Updater manifest is missing platforms");
  }

  for (const [platform, matcher] of Object.entries(PLATFORM_ARTIFACTS)) {
    const entry = manifest.platforms[platform];
    if (!entry) continue;
    if (!entry.signature) throw new Error(`Updater signature is missing for ${platform}`);

    const matches = assetNames.filter((name) => matcher.test(name));
    if (matches.length !== 1) {
      throw new Error(`Expected one updater artifact for ${platform}, found ${matches.length}`);
    }
    entry.url = releaseAssetUrl(repository, tag, matches[0]);
  }

  for (const platform of ["darwin-aarch64", "windows-x86_64"]) {
    const entry = manifest.platforms[platform];
    if (!entry?.url || !entry.signature) throw new Error(`Missing updater platform: ${platform}`);
    if (new URL(entry.url).hostname !== "github.com") {
      throw new Error(`Updater URL must use the public GitHub release host: ${platform}`);
    }
  }

  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  return manifest;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [manifestPath, assetsPath, repository, tag] = process.argv.slice(2);
  if (!manifestPath || !assetsPath || !repository || !tag) {
    throw new Error("Usage: node scripts/rewrite-updater-manifest.mjs <manifest> <assets> <owner/repo> <tag>");
  }
  const manifest = await rewriteUpdaterManifest({ manifestPath, assetsPath, repository, tag });
  for (const [platform, entry] of Object.entries(manifest.platforms)) {
    console.log(`${platform}: ${entry.url}`);
  }
}
