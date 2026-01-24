import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const od = path.join(__dirname, 'output');
const dirs = fs.readdirSync(od);

// Category 1: Fragment.nodes array length mismatch (parser stops early)
console.log("=== FRAGMENT LENGTH MISMATCHES (parser stops early) ===\n");
let c = 0;
for (const d of dirs) {
  if (c >= 5) break;
  const sf = path.join(od, d, 'svelte.json');
  const lf = path.join(od, d, 'lux.json');
  if (!fs.existsSync(sf) || !fs.existsSync(lf)) continue;
  const s = JSON.parse(fs.readFileSync(sf, 'utf8'));
  const l = JSON.parse(fs.readFileSync(lf, 'utf8'));
  if (!s.fragment || !l.fragment) continue;
  const sn = s.fragment.nodes || [];
  const ln = l.fragment.nodes || [];
  if (sn.length > ln.length && sn.length <= 6) {
    console.log(`${d}: expected ${sn.length} nodes, got ${ln.length}`);
    console.log(`  expected: [${sn.map(n => n.type).join(', ')}]`);
    console.log(`  got:      [${ln.map(n => n.type).join(', ')}]`);
    console.log();
    c++;
  }
}

// Category 2: Enum casing issues
console.log("\n=== ENUM CASING ISSUES ===\n");
c = 0;
for (const d of dirs) {
  if (c >= 3) break;
  const sf = path.join(od, d, 'svelte.json');
  const lf = path.join(od, d, 'lux.json');
  if (!fs.existsSync(sf) || !fs.existsSync(lf)) continue;
  const s = JSON.parse(fs.readFileSync(sf, 'utf8'));
  const l = JSON.parse(fs.readFileSync(lf, 'utf8'));
  if (l.css === "Injected" && s.css === "injected") {
    console.log(`${d}: css="${l.css}" should be "${s.css}"`);
    c++;
  }
  if (l.namespace && s.namespace && l.namespace !== s.namespace) {
    console.log(`${d}: namespace="${l.namespace}" should be "${s.namespace}"`);
    c++;
  }
}

// Category 3: SvelteElement vs RegularElement
console.log("\n=== SVELTE:ELEMENT MISIDENTIFICATION ===\n");
c = 0;
function findNodeType(nodes, targetType) {
  for (const n of (nodes || [])) {
    if (n.type === targetType) return n;
    if (n.fragment && n.fragment.nodes) {
      const found = findNodeType(n.fragment.nodes, targetType);
      if (found) return found;
    }
  }
  return null;
}
for (const d of dirs) {
  if (c >= 3) break;
  const sf = path.join(od, d, 'svelte.json');
  const lf = path.join(od, d, 'lux.json');
  if (!fs.existsSync(sf) || !fs.existsSync(lf)) continue;
  const s = JSON.parse(fs.readFileSync(sf, 'utf8'));
  const l = JSON.parse(fs.readFileSync(lf, 'utf8'));
  const svelteEl = findNodeType(s.fragment?.nodes, 'SvelteElement');
  if (svelteEl) {
    const luxEl = findNodeType(l.fragment?.nodes, 'RegularElement');
    if (luxEl && luxEl.name === svelteEl.name) {
      console.log(`${d}: "${svelteEl.name}" parsed as RegularElement, should be SvelteElement`);
      c++;
    }
  }
}

// Category 4: leadingComments missing
console.log("\n=== LEADING COMMENTS MISSING ===\n");
c = 0;
function findMissingComments(sNode, lNode, path) {
  if (!sNode || !lNode) return [];
  const results = [];
  if (sNode.leadingComments && !lNode.leadingComments) {
    results.push(`${path}: missing leadingComments (${sNode.leadingComments.length} comments)`);
  }
  if (sNode.trailingComments && !lNode.trailingComments) {
    results.push(`${path}: missing trailingComments (${sNode.trailingComments.length} comments)`);
  }
  // Check body array
  if (Array.isArray(sNode.body) && Array.isArray(lNode.body)) {
    for (let i = 0; i < Math.min(sNode.body.length, lNode.body.length); i++) {
      results.push(...findMissingComments(sNode.body[i], lNode.body[i], `${path}.body[${i}]`));
    }
  }
  return results;
}
for (const d of dirs) {
  if (c >= 5) break;
  const sf = path.join(od, d, 'svelte.json');
  const lf = path.join(od, d, 'lux.json');
  if (!fs.existsSync(sf) || !fs.existsSync(lf)) continue;
  const s = JSON.parse(fs.readFileSync(sf, 'utf8'));
  const l = JSON.parse(fs.readFileSync(lf, 'utf8'));

  // Check instance script
  if (s.instance?.content && l.instance?.content) {
    const issues = findMissingComments(s.instance.content, l.instance.content, 'instance.content');
    if (issues.length > 0) {
      console.log(`${d}:`);
      issues.forEach(i => console.log(`  ${i}`));
      c++;
    }
  }
}

// Category 5: OXC extra fields still present
console.log("\n=== OXC EXTRAS STILL PRESENT (strip_oxc_extras gaps) ===\n");
c = 0;
function findOxcExtras(node, path) {
  if (!node || typeof node !== 'object') return [];
  const results = [];
  const oxcFields = ['decorators', 'definite', 'abstract', 'declare', 'accessibility', 'override'];
  for (const field of oxcFields) {
    if (field in node && node[field] !== undefined) {
      results.push(`${path}.${field} = ${JSON.stringify(node[field])}`);
    }
  }
  for (const [key, val] of Object.entries(node)) {
    if (Array.isArray(val)) {
      val.forEach((item, i) => results.push(...findOxcExtras(item, `${path}.${key}[${i}]`)));
    } else if (typeof val === 'object' && val !== null) {
      results.push(...findOxcExtras(val, `${path}.${key}`));
    }
  }
  return results;
}
for (const d of dirs) {
  if (c >= 3) break;
  const lf = path.join(od, d, 'lux.json');
  if (!fs.existsSync(lf)) continue;
  const l = JSON.parse(fs.readFileSync(lf, 'utf8'));
  const extras = findOxcExtras(l, 'root');
  if (extras.length > 0) {
    console.log(`${d}: ${extras.length} OXC extras`);
    extras.slice(0, 3).forEach(e => console.log(`  ${e}`));
    c++;
  }
}
