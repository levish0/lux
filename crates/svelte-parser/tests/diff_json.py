"""Compare two JSON files and print structural differences."""
import json
import sys

def diff_json(a, b, path='root', diffs=None):
    if diffs is None:
        diffs = []
    if type(a) != type(b):
        diffs.append(f'{path}: type {type(a).__name__} vs {type(b).__name__}')
        return diffs
    if isinstance(a, dict):
        for k in sorted(set(list(a.keys()) + list(b.keys()))):
            if k not in a:
                diffs.append(f'{path}.{k}: MISSING in lux')
            elif k not in b:
                diffs.append(f'{path}.{k}: EXTRA in lux = {json.dumps(a[k])[:80]}')
            else:
                diff_json(a[k], b[k], f'{path}.{k}', diffs)
    elif isinstance(a, list):
        if len(a) != len(b):
            diffs.append(f'{path}: list len {len(a)} vs {len(b)}')
        for i in range(min(len(a), len(b))):
            diff_json(a[i], b[i], f'{path}[{i}]', diffs)
    else:
        if a != b:
            diffs.append(f'{path}: {json.dumps(a)[:60]} vs {json.dumps(b)[:60]}')
    return diffs

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: python diff_json.py <lux.json> <svelte.json> [max_diffs]")
        sys.exit(1)
    with open(sys.argv[1]) as f:
        lux = json.load(f)
    with open(sys.argv[2]) as f:
        svelte = json.load(f)
    max_diffs = int(sys.argv[3]) if len(sys.argv) > 3 else 30
    diffs = diff_json(lux, svelte)
    for d in diffs[:max_diffs]:
        print(d)
    if len(diffs) > max_diffs:
        print(f'... ({len(diffs) - max_diffs} more)')
    print(f'\nTotal: {len(diffs)} differences')
