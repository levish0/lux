import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(__dirname, 'output');
const dirs = fs.readdirSync(outDir).filter(d => {
    try {
        const l = JSON.parse(fs.readFileSync(path.join(outDir, d, 'lux.json'), 'utf8'));
        const s = JSON.parse(fs.readFileSync(path.join(outDir, d, 'svelte.json'), 'utf8'));
        return JSON.stringify(l) !== JSON.stringify(s);
    } catch(e) { return false; }
}).slice(0, 3);

for (const d of dirs) {
    console.log('\n=== ' + d + ' ===');
    const l = JSON.parse(fs.readFileSync(path.join(outDir, d, 'lux.json'), 'utf8'));
    const s = JSON.parse(fs.readFileSync(path.join(outDir, d, 'svelte.json'), 'utf8'));

    // Find first difference
    function findDiff(a, b, path) {
        if (a === b) return null;
        if (a === null || b === null) return path + ': ' + a + ' vs ' + b;
        if (typeof a !== typeof b) return path + ': type ' + typeof a + ' vs ' + typeof b;
        if (typeof a !== 'object') return path + ': ' + JSON.stringify(a) + ' vs ' + JSON.stringify(b);
        if (Array.isArray(a) !== Array.isArray(b)) return path + ': array vs object';
        if (Array.isArray(a)) {
            if (a.length !== b.length) return path + ': len ' + a.length + ' vs ' + b.length;
            for (let i = 0; i < a.length; i++) {
                const r = findDiff(a[i], b[i], path + '[' + i + ']');
                if (r) return r;
            }
            return null;
        }
        const allKeys = new Set([...Object.keys(a), ...Object.keys(b)]);
        for (const k of allKeys) {
            if (!(k in a)) return path + ': MISSING ' + k;
            if (!(k in b)) return path + ': EXTRA ' + k;
            const r = findDiff(a[k], b[k], path + '.' + k);
            if (r) return r;
        }
        return null;
    }
    console.log(findDiff(l, s, 'root'));
}
