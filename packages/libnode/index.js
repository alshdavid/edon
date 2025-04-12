import syncFs, { fsyncSync } from "node:fs";
import { cpus } from "node:os";
import fs from "node:fs/promises";
import path from "node:path";
import url from "node:url";
import { spawn } from "node:child_process";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url))

const nodejsGithubRepo = "https://github.com/nodejs/node";

const OS = process.env.OS || { "darwin": "mac", "win32": "windows" }[process.platform] || process.platform;
const ARCH = process.env.ARCH || process.arch;

const coreCount = cpus().length;
const threadCount = coreCount * 2;

const spawnAsync = (program, args) =>
  new Promise((resolve, reject) => {
    console.log("Running:", [program, ...args].join(" "));

    const child = spawn(program, args, { shell: true });

    child.stdout.on("data", (chunk) => console.log(chunk.toString()));
    child.stderr.on("data", (chunk) => console.error(chunk.toString()));
    child.on("close", (code) => {
      if (code == 0) resolve(code.toString());
      else reject(code.toString());
    });
});

const version = await fs.readFile("version.txt", { encoding: "utf8" });
if (!syncFs.existsSync("node")) {
  await spawnAsync(
    "git",
    ["clone", nodejsGithubRepo, "--branch", version, "--depth=1"],
    undefined,
    {}
  );
}

const wrapper = syncFs.readFileSync(path.join(__dirname, 'wrapper','node.cc'), 'utf8')
const originalPath = path.join(__dirname, 'node', 'src', 'node.cc')
const original = syncFs.readFileSync(originalPath, 'utf8')

syncFs.appendFileSync(
  originalPath,
  "\n\n" + wrapper,
  'utf8'
)

try {
  process.chdir("node");

  let extraArgs = [];
  if (process.platform == "win32") {
    await spawnAsync(".\\vcbuild.bat", [ARCH, "dll", "openssl-no-asm"]);
    syncFs.writeFileSync(originalPath, original, 'utf8')
    process.exit()
  } 

  if (ARCH === "arm64") {
    extraArgs.push("--with-arm-float-abi");
    extraArgs.push("hard");
    extraArgs.push("--with-arm-fpu");
    extraArgs.push("neon");
  }

  await spawnAsync("./configure", [
    "--shared",
    "--dest-cpu",
    ARCH,
    "--dest-os",
    OS,
    ...extraArgs,
  ]);


  // await spawnAsync("make", [`-j${threadCount}`]);
  // syncFs.writeFileSync(originalPath, original, 'utf8')
} catch (error) {
  syncFs.writeFileSync(originalPath, original, 'utf8') 
}