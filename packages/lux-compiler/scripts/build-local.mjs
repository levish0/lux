import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const isRelease = process.argv.includes("--release");
const profile = isRelease ? "release" : "debug";
const workspaceRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const packageRoot = resolve(workspaceRoot, "packages/lux-compiler");
const explicitOutIndex = process.argv.indexOf("--out");
const explicitOutPath =
  explicitOutIndex >= 0 ? process.argv[explicitOutIndex + 1] : undefined;

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

const targetNode = resolve(
  explicitOutPath || process.env.LUX_NODE_BINDING_OUT || resolve(packageRoot, "lux_node.node"),
);
mkdirSync(dirname(targetNode), { recursive: true });
copyFileSync(sourceLib, targetNode);
console.log(`Copied ${sourceLib} -> ${targetNode}`);

function libFileName() {
  if (process.platform === "win32") return "lux_node.dll";
  if (process.platform === "darwin") return "liblux_node.dylib";
  return "liblux_node.so";
}
