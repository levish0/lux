import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const isRelease = process.argv.includes("--release");
const profile = isRelease ? "release" : "debug";
const workspaceRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const packageRoot = resolve(workspaceRoot, "packages/lux-compiler");

const cargoArgs = ["build", "-p", "lux-node"];
if (isRelease) cargoArgs.push("--release");

const cargo =
  process.platform === "win32"
    ? spawnSync("cmd.exe", ["/d", "/s", "/c", "cargo", ...cargoArgs], {
        cwd: workspaceRoot,
        stdio: "inherit",
      })
    : spawnSync("cargo", cargoArgs, {
        cwd: workspaceRoot,
        stdio: "inherit",
      });
if (cargo.status !== 0) {
  process.exit(cargo.status ?? 1);
}

const sourceLib = resolve(workspaceRoot, "target", profile, libFileName());
if (!existsSync(sourceLib)) {
  throw new Error(`Built library not found: ${sourceLib}`);
}

const targetNode = resolve(packageRoot, "lux_node.node");
copyFileSync(sourceLib, targetNode);
console.log(`Copied ${sourceLib} -> ${targetNode}`);

function libFileName() {
  if (process.platform === "win32") return "lux_node.dll";
  if (process.platform === "darwin") return "liblux_node.dylib";
  return "liblux_node.so";
}
