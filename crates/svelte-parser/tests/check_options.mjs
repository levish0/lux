import fs from 'fs';
import path from 'path';

const outputDir = 'tests/output';
const dirs = fs.readdirSync(outputDir).sort();
let count = 0;
for (const dir of dirs) {
  const sp = path.join(outputDir, dir, 'svelte.json');
  const lp = path.join(outputDir, dir, 'lux.json');
  if (!(fs.existsSync(sp) && fs.existsSync(lp))) continue;
  const e = JSON.parse(fs.readFileSync(sp, 'utf8'));
  const a = JSON.parse(fs.readFileSync(lp, 'utf8'));
  if (e.options && (e.options.customElement || e.options.namespace || e.options.css)) {
    const missing = [];
    if (e.options.customElement && !(a.options && a.options.customElement)) missing.push('customElement');
    if (e.options.namespace && !(a.options && a.options.namespace)) missing.push('namespace');
    if (e.options.css && !(a.options && a.options.css)) missing.push('css');
    if (missing.length > 0) {
      if (count < 5) {
        console.log(dir, '- missing:', missing.join(', '));
        console.log('  Expected:', JSON.stringify(e.options).substring(0, 400));
        if (a.options) console.log('  Actual:', JSON.stringify(a.options).substring(0, 400));
      }
      count++;
    }
  }
}
console.log('\nTotal options issues:', count);
