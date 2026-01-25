import fs from 'fs';
import path from 'path';

const SKIP_FIELDS = ['start', 'end', 'loc', 'leadingComments', 'trailingComments'];

function deepCompare(actual, expected, path = '') {
    const diffs = [];

    if (expected === null || expected === undefined) {
        return diffs;
    }

    if (typeof expected !== typeof actual) {
        diffs.push({ path, type: 'type_mismatch', expected: typeof expected, actual: typeof actual, expectedVal: expected, actualVal: actual });
        return diffs;
    }

    if (Array.isArray(expected)) {
        if (!Array.isArray(actual)) {
            diffs.push({ path, type: 'not_array', expected: 'array', actual: typeof actual });
            return diffs;
        }
        if (actual.length !== expected.length) {
            diffs.push({ path, type: 'array_length', expected: expected.length, actual: actual.length });
        }
        const minLen = Math.min(actual.length, expected.length);
        for (let i = 0; i < minLen; i++) {
            diffs.push(...deepCompare(actual[i], expected[i], `${path}[${i}]`));
        }
        return diffs;
    }

    if (typeof expected === 'object') {
        if (typeof actual !== 'object' || actual === null) {
            diffs.push({ path, type: 'not_object', expected: 'object', actual: actual });
            return diffs;
        }
        for (const key of Object.keys(expected)) {
            if (SKIP_FIELDS.includes(key)) continue;

            if (!(key in actual)) {
                diffs.push({ path: `${path}.${key}`, type: 'missing_field', expected: expected[key] });
            } else {
                diffs.push(...deepCompare(actual[key], expected[key], `${path}.${key}`));
            }
        }
        return diffs;
    }

    // Primitives
    if (actual !== expected) {
        diffs.push({ path, type: 'value_mismatch', expected, actual });
    }

    return diffs;
}

const outputDir = path.join(import.meta.dirname, 'output');

const mismatches = [
    'compiler-errors--snippet-rest-args--main',
    'css--quote-mark-inside-string--input',
    'migrate--accessors--output',
    'migrate--named-slots--output',
    'migrate--slot-use_ts--output',
    'migrate--slot-use_ts-2--output',
    'parser-modern--generic-snippets--input',
    'parser-modern--loose-invalid-expression--input',
    'parser-modern--loose-unclosed-open-tag--input',
    'parser-modern--loose-unclosed-tag--input',
    'runtime-runes--each-dynamic-html--main',
    'runtime-runes--typescript--main',
    'sourcemaps--typescript--input',
    'validator--silence-warnings--input',
    'validator--ts-unsupported-accessor--input',
    'validator--ts-unsupported-enum--input',
];

for (const name of mismatches) {
    const dir = path.join(outputDir, name);
    const luxPath = path.join(dir, 'lux.json');
    const sveltePath = path.join(dir, 'svelte.json');

    if (!fs.existsSync(luxPath) || !fs.existsSync(sveltePath)) {
        console.log(`\n=== ${name} ===`);
        console.log('Missing files');
        continue;
    }

    const lux = JSON.parse(fs.readFileSync(luxPath, 'utf-8'));
    const svelte = JSON.parse(fs.readFileSync(sveltePath, 'utf-8'));

    const diffs = deepCompare(lux, svelte);

    if (diffs.length > 0) {
        console.log(`\n=== ${name} ===`);
        // Show first 5 diffs only
        for (const diff of diffs.slice(0, 5)) {
            console.log(`  ${diff.path}: ${diff.type}`);
            if (diff.type === 'value_mismatch') {
                console.log(`    expected: ${JSON.stringify(diff.expected)}`);
                console.log(`    actual:   ${JSON.stringify(diff.actual)}`);
            } else if (diff.type === 'missing_field') {
                console.log(`    expected: ${JSON.stringify(diff.expected).slice(0, 100)}`);
            } else if (diff.type === 'array_length') {
                console.log(`    expected: ${diff.expected}, actual: ${diff.actual}`);
            }
        }
        if (diffs.length > 5) {
            console.log(`  ... and ${diffs.length - 5} more differences`);
        }
    }
}
