import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

const docsDir = fileURLToPath(new URL("../docs", import.meta.url));
const useInstall = process.argv.includes("--install");
const passthroughArgs = process.argv.filter((arg) => arg !== "--install").slice(2);

if (!existsSync(docsDir)) {
  console.error(`missing docs directory: ${docsDir}`);
  process.exit(2);
}

const env = { ...process.env, LUX_SVELTE: "1" };

runPackageManager(["install", "--frozen-lockfile"], { run: useInstall, env });
runPackageManager(["build", ...passthroughArgs], { run: true, env });

function runPackageManager(args, { run, env }) {
  if (!run) return;

  const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
  const pnpmRun = spawnSync(pnpm, args, {
    cwd: docsDir,
    stdio: "inherit",
    env,
  });
  if (!pnpmRun.error) {
    process.exitCode = pnpmRun.status ?? 1;
    if (process.exitCode !== 0) process.exit(process.exitCode);
    return;
  }

  if (pnpmRun.error.code !== "ENOENT") {
    console.error(pnpmRun.error);
    process.exit(1);
  }

  const npm = process.platform === "win32" ? "npm.cmd" : "npm";
  const npmArgs = args[0] === "build" ? ["run", "build", "--", ...args.slice(1)] : args;
  const npmRun = spawnSync(npm, npmArgs, {
    cwd: docsDir,
    stdio: "inherit",
    env,
  });

  process.exitCode = npmRun.status ?? 1;
  if (process.exitCode !== 0) {
    if (npmRun.error) console.error(npmRun.error);
    process.exit(process.exitCode);
  }
}
