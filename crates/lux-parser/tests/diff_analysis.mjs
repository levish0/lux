import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(__dirname, 'output');
const dirs = fs.readdirSync(outDir);

let diffs = {};
let count = 0;

// Check if lux is a superset of svelte (svelte's fields are all in lux with matching values)
function isSuperset(lux, svelte) {
    if (lux === svelte) return true;
    if (svelte === null || svelte === undefined) return true;
    if (lux === null || lux === undefined) return false;
    if (typeof lux !== typeof svelte) return false;
    if (Array.isArray(svelte)) {
        if (!Array.isArray(lux)) return false;
        if (lux.length !== svelte.length) return false;
        return lux.every((item, i) => isSuperset(item, svelte[i]));
    }
    if (typeof svelte === 'object') {
        for (const k of Object.keys(svelte)) {
            if (!(k in lux)) return false;
            if (!isSuperset(lux[k], svelte[k])) return false;
        }
        return true;
    }
    return lux === svelte;
}

// Report only MISSING fields and wrong values (not EXTRA)
function diffNodes(lux, svelte, nodePath) {
    if (lux === null && svelte === null) return;
    if (svelte === null || svelte === undefined) return; // extra in lux, fine
    if (lux === null || lux === undefined) {
        const key = nodePath + ': null_vs_value';
        diffs[key] = (diffs[key] || 0) + 1;
        return;
    }
    if (typeof lux !== typeof svelte) {
        const key = nodePath + ': type_mismatch(' + typeof lux + ' vs ' + typeof svelte + ')';
        diffs[key] = (diffs[key] || 0) + 1;
        return;
    }
    if (Array.isArray(lux) && Array.isArray(svelte)) {
        if (lux.length !== svelte.length) {
            const key = nodePath + ': array_length(' + lux.length + ' vs ' + svelte.length + ')';
            diffs[key] = (diffs[key] || 0) + 1;
            return;
        }
        for (let i = 0; i < lux.length; i++) {
            diffNodes(lux[i], svelte[i], nodePath + '[' + i + ']');
        }
        return;
    }
    if (typeof lux === 'object') {
        const svelteKeys = Object.keys(svelte);
        for (const k of svelteKeys) {
            if (!(k in lux)) {
                const nodeType = svelte.type || '';
                const key = 'MISSING:' + k + ' (in ' + nodeType + ')';
                diffs[key] = (diffs[key] || 0) + 1;
            } else if (JSON.stringify(lux[k]) !== JSON.stringify(svelte[k])) {
                if (k === 'start' || k === 'end') continue; // skip offset diffs
                diffNodes(lux[k], svelte[k], (lux.type || svelte.type || '') + '.' + k);
            }
        }
        return;
    }
    if (lux !== svelte) {
        const key = nodePath + ': ' + JSON.stringify(lux) + ' vs ' + JSON.stringify(svelte);
        diffs[key] = (diffs[key] || 0) + 1;
    }
}

for (const dir of dirs) {
    const luxPath = path.join(outDir, dir, 'lux.json');
    const sveltePath = path.join(outDir, dir, 'svelte.json');
    if (!fs.existsSync(luxPath) || !fs.existsSync(sveltePath)) continue;
    const lux = JSON.parse(fs.readFileSync(luxPath, 'utf8'));
    const svelte = JSON.parse(fs.readFileSync(sveltePath, 'utf8'));
    if (isSuperset(lux, svelte)) continue;
    count++;
    diffNodes(lux, svelte, 'root');
}

console.log('Files with diffs:', count);
console.log('');
const sorted = Object.entries(diffs).sort((a, b) => b[1] - a[1]).slice(0, 50);
for (const [key, val] of sorted) {
    console.log(val + '\t' + key);
}
