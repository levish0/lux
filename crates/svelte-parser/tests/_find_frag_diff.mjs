import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(__dirname, 'output');
const dirs = fs.readdirSync(outDir).sort();

// Find cases where lux=2, svelte=3
let examples23 = [];
// Find cases where lux > svelte (error recovery adding extra)
let examplesMore = [];

for (const dir of dirs) {
    const luxPath = path.join(outDir, dir, 'lux.json');
    const sveltePath = path.join(outDir, dir, 'svelte.json');
    if (!fs.existsSync(luxPath) || !fs.existsSync(sveltePath)) continue;
    const l = JSON.parse(fs.readFileSync(luxPath, 'utf8'));
    const s = JSON.parse(fs.readFileSync(sveltePath, 'utf8'));
    if (l.fragment && s.fragment && l.fragment.nodes && s.fragment.nodes) {
        const ll = l.fragment.nodes.length;
        const sl = s.fragment.nodes.length;
        if (ll === 2 && sl === 3 && examples23.length < 3) {
            examples23.push(dir);
        }
        if (ll > sl && examplesMore.length < 3) {
            examplesMore.push({ dir, lux: ll, svelte: sl });
        }
    }
}

console.log("=== lux=2 vs svelte=3 ===");
for (const ex of examples23) {
    console.log(ex);
}
console.log("\n=== lux > svelte (error recovery extras) ===");
for (const ex of examplesMore) {
    console.log(`${ex.dir}: lux=${ex.lux} svelte=${ex.svelte}`);
}
