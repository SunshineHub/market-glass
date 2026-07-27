import assert from "node:assert/strict";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { rewriteUpdaterManifest } from "./rewrite-updater-manifest.mjs";

test("rewrites GitHub API asset URLs to public release downloads", async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), "market-glass-updater-"));
  const manifestPath = path.join(directory, "latest.json");
  const assetsPath = path.join(directory, "assets.json");
  const signature = "signed";

  await writeFile(manifestPath, JSON.stringify({
    version: "0.1.10",
    platforms: {
      "darwin-aarch64": { signature, url: "https://api.github.com/repos/example/app/releases/assets/1" },
      "darwin-aarch64-app": { signature, url: "https://api.github.com/repos/example/app/releases/assets/1" },
      "windows-x86_64": { signature, url: "https://api.github.com/repos/example/app/releases/assets/2" },
      "windows-x86_64-nsis": { signature, url: "https://api.github.com/repos/example/app/releases/assets/2" },
    },
  }));
  await writeFile(assetsPath, JSON.stringify({ assets: [
    { name: "Market.Glass_0.1.10_aarch64.app.tar.gz" },
    { name: "Market.Glass_0.1.10_aarch64.app.tar.gz.sig" },
    { name: "Market.Glass_0.1.10_aarch64.dmg" },
    { name: "Market.Glass_0.1.10_x64-setup.exe" },
    { name: "Market.Glass_0.1.10_x64-setup.exe.sig" },
  ] }));

  const result = await rewriteUpdaterManifest({
    manifestPath,
    assetsPath,
    repository: "SunshineHub/market-glass",
    tag: "v0.1.10",
  });

  assert.equal(
    result.platforms["darwin-aarch64"].url,
    "https://github.com/SunshineHub/market-glass/releases/download/v0.1.10/Market.Glass_0.1.10_aarch64.app.tar.gz",
  );
  assert.equal(
    result.platforms["windows-x86_64"].url,
    "https://github.com/SunshineHub/market-glass/releases/download/v0.1.10/Market.Glass_0.1.10_x64-setup.exe",
  );
  assert.deepEqual(JSON.parse(await readFile(manifestPath, "utf8")), result);
});

test("fails when a required updater artifact is ambiguous", async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), "market-glass-updater-"));
  const manifestPath = path.join(directory, "latest.json");
  const assetsPath = path.join(directory, "assets.json");

  await writeFile(manifestPath, JSON.stringify({
    platforms: { "darwin-aarch64": { signature: "signed", url: "https://api.github.com/asset/1" } },
  }));
  await writeFile(assetsPath, JSON.stringify({ assets: [] }));

  await assert.rejects(
    rewriteUpdaterManifest({ manifestPath, assetsPath, repository: "example/app", tag: "v1.0.0" }),
    /Expected one updater artifact/,
  );
});
