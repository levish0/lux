import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
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

  const pnpmRun = runCommand(resolveExecutable("pnpm"), args, env);

  if (!pnpmRun.error && pnpmRun.status === 0) {
    process.exitCode = pnpmRun.status ?? 1;
    if (process.exitCode !== 0) process.exit(process.exitCode);
    return;
  }

  if (!pnpmRun.error) {
    process.exit(pnpmRun.status ?? 1);
  }

  if (pnpmRun.error.code !== "ENOENT") {
    console.error(pnpmRun.error);
    process.exit(1);
  }

  const npmArgs = args[0] === "build" ? ["run", "build", "--", ...args.slice(1)] : args;
  const npmRun = runCommand(resolveExecutable("npm"), npmArgs, env);

  if (npmRun.error) {
    console.error(npmRun.error);
    process.exit(1);
  }

  process.exitCode = npmRun.status ?? 0;
  if (process.exitCode !== 0) {
    process.exit(process.exitCode);
  }
}

function resolveExecutable(name) {
  if (process.platform !== "win32") return name;
  const cmdPath = join(dirname(process.execPath), `${name}.cmd`);
  return existsSync(cmdPath) ? cmdPath : name;
}

function runCommand(command, args, env) {
  const options = {
    cwd: docsDir,
    stdio: "inherit",
    env,
  };

  const result = spawnSync(command, args, options);
  if (process.platform !== "win32" || result.error?.code !== "EINVAL") {
    return result;
  }

  return spawnSync("cmd.exe", ["/d", "/s", "/c", command, ...args], options);
}
