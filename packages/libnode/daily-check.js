import fs from "node:fs/promises";
import { execSync } from "node:child_process";

const nodejsGithubRepo = "https://github.com/nodejs/node";
const removeTheVCharacter = (str) => str.replace("v", "");

const nodeIndexUrl = "https://nodejs.org/dist/index.json";
const getLatestNodeVersion = async () => {
  const res = await fetch(nodeIndexUrl);
  const jsonData = await res.json();

  return removeTheVCharacter(jsonData[0]["version"]);
};

const getLatestPublishedVersion = async () =>
  removeTheVCharacter(await fs.readFile("version.txt", { encoding: "utf8" }));

const isANewerVersion = (oldVer, newVer) => {
  const oldParts = oldVer.split(".");
  const newParts = newVer.split(".");

  for (var i = 0; i < newParts.length; i++) {
    const a = ~~newParts[i]; // parse int
    const b = ~~oldParts[i]; // parse int

    if (a > b) return true;
    if (a < b) return false;
  }

  return false;
};

const latestNodeVersion = await getLatestNodeVersion();
const latestPublishedVersion = await getLatestPublishedVersion();
if (!isANewerVersion(latestPublishedVersion, latestNodeVersion)) {
  console.log("Nothing to do!");
  process.exit(0);
}

execSync(`echo "NOTHING_TO_DO=false" >> $GITHUB_ENV`);
execSync(`echo "TAG=v${latestNodeVersion}" >> $GITHUB_ENV`);

await fs.writeFile("version.txt", `v${latestNodeVersion}`);
