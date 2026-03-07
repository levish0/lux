import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const toolDir = dirname(fileURLToPath(import.meta.url));
const workspaceRoot = resolve(toolDir, "..");
const docsSourceDir = resolve(workspaceRoot, "docs");
const localCompilerDir = resolve(workspaceRoot, "packages/lux-compiler");
const localPluginDir = resolve(workspaceRoot, "packages/vite-plugin-svelte-lux");
const tempBaseDir = resolve(workspaceRoot, "docs_lux");
const runId = new Date().toISOString().replace(/[:.]/g, "-");
const tempRootDir = resolve(tempBaseDir, runId);
const tempDocsDir = resolve(tempRootDir, "docs");
const tempBindingPath = resolve(tempRootDir, "lux_node.node");
const cleanTemp = process.argv.includes("--clean-temp");
const passthroughArgs = process.argv.filter((arg) => arg !== "--clean-temp").slice(2);

if (!existsSync(docsSourceDir)) {
	console.error(`missing docs directory: ${docsSourceDir}`);
	process.exit(2);
}
if (!existsSync(localCompilerDir)) {
	console.error(`missing local compiler package: ${localCompilerDir}`);
	process.exit(2);
}
if (!existsSync(localPluginDir)) {
	console.error(`missing local plugin package: ${localPluginDir}`);
	process.exit(2);
}

const env = {
	...process.env,
	LUX_SVELTE: "1",
	LUX_NODE_BINDING: tempBindingPath,
	LUX_NODE_BINDING_OUT: tempBindingPath,
	LUX_ARTIFACTS_DIR: resolve(tempDocsDir, ".lux_artifacts")
};

if (cleanTemp) {
	rmSync(tempBaseDir, { recursive: true, force: true });
}

runNodeScript(resolve(localCompilerDir, "scripts/build-local.mjs"), env, workspaceRoot);
prepareTempDocsCopy();
patchTempDocsPackageJson();
runPnpmOrExit(["install", "--ignore-scripts"], env, tempDocsDir);

console.log(`Lux docs dev started from temp copy: ${tempDocsDir}`);
console.log(`Artifacts dir: ${resolve(tempDocsDir, ".lux_artifacts")}`);
console.log("Press Ctrl+C to stop.");

runPnpmOrExit(["dev", ...passthroughArgs], env, tempDocsDir);

function prepareTempDocsCopy() {
	mkdirSync(tempRootDir, { recursive: true });
	cpSync(docsSourceDir, tempDocsDir, {
		recursive: true,
		force: true,
		filter: (sourcePath) => {
			const name = basename(sourcePath);
			return name !== "node_modules" && name !== ".svelte-kit" && name !== ".wrangler";
		}
	});
}

function patchTempDocsPackageJson() {
	const packageJsonPath = resolve(tempDocsDir, "package.json");
	const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
	packageJson.devDependencies ??= {};
	packageJson.devDependencies["@sveltejs/vite-plugin-svelte"] = toFileSpecifier(
		tempDocsDir,
		localPluginDir
	);
	writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, "\t")}\n`);
}

function toFileSpecifier(fromDir, targetDir) {
	let rel = relative(fromDir, targetDir).replace(/\\/g, "/");
	if (!rel.startsWith(".")) {
		rel = `./${rel}`;
	}
	return `file:${rel}`;
}

function runNodeScript(scriptPath, env, cwd) {
	const run = runCommand(process.execPath, [scriptPath], env, cwd);
	if (run.error) {
		console.error(run.error);
		process.exit(1);
	}
	if ((run.status ?? 0) !== 0) {
		process.exit(run.status ?? 1);
	}
}

function runPnpmOrExit(args, env, cwd) {
	const run = runPnpm(args, env, cwd);
	if (run.error) {
		console.error(run.error);
		process.exit(1);
	}
	if ((run.status ?? 0) !== 0) {
		process.exit(run.status ?? 1);
	}
}

function runPnpm(args, env, cwd) {
	const direct = runCommand(resolveExecutable("pnpm"), args, env, cwd);
	if (!direct.error || direct.error.code !== "ENOENT") {
		return direct;
	}
	return runCommand(resolveExecutable("corepack"), ["pnpm", ...args], env, cwd);
}

function resolveExecutable(name) {
	if (process.platform !== "win32") return name;
	const cmdPath = join(dirname(process.execPath), `${name}.cmd`);
	return existsSync(cmdPath) ? cmdPath : name;
}

function runCommand(command, args, env, cwd) {
	const options = { cwd, stdio: "inherit", env };
	const result = spawnSync(command, args, options);
	if (process.platform !== "win32" || result.error?.code !== "EINVAL") {
		return result;
	}
	return spawnSync("cmd.exe", ["/d", "/s", "/c", command, ...args], options);
}
