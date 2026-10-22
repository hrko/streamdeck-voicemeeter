import { readdir, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

export function flatten(value, prefix = '') {
  return Object.fromEntries(Object.entries(value).flatMap(([key, item]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    return typeof item === 'string' ? [[path, item]] : Object.entries(flatten(item, path));
  }));
}

const variables = text => [...text.matchAll(/{{\s*([^},\s]+)(?:[^}]*)}}/g)].map(match => match[1]).sort().join(',');
export function checkCatalog(base, candidate, complete) {
  const reference = flatten(base), translated = flatten(candidate), errors = [];
  for (const [key, value] of Object.entries(translated)) {
    if (!(key in reference)) errors.push(`Unknown translation key: ${key}`);
    else if (variables(value) !== variables(reference[key])) errors.push(`Interpolation mismatch: ${key}`);
  }
  const missing = Object.keys(reference).filter(key => !(key in translated));
  if (complete) errors.push(...missing.map(key => `Missing translation: ${key}`));
  return { errors, missing };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const directory = new URL('../pi/src/locales/', import.meta.url);
  const files = (await readdir(directory)).filter(file => file.endsWith('.json'));
  const base = JSON.parse(await readFile(new URL('en.json', directory), 'utf8'));
  let failed = false;
  for (const file of files) {
    const catalog = JSON.parse(await readFile(new URL(file, directory), 'utf8'));
    const result = checkCatalog(base, catalog, ['en.json', 'ja.json'].includes(file));
    for (const error of result.errors) { console.error(`${file}: ${error}`); failed = true; }
    if (result.missing.length && !['en.json', 'ja.json'].includes(file)) console.log(`${file}: ${result.missing.length} keys use English fallback`);
  }
  if (!files.includes('ja.json')) { console.error('Missing required ja.json'); failed = true; }
  if (failed) process.exitCode = 1;
  else console.log(`Validated ${files.length} locale catalogs`);
}
