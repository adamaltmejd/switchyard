// Image-build only. npm has already checked the lockfile's archive integrities.
import { readdir, readFile, writeFile, lstat } from 'node:fs/promises';
import { createHash } from 'node:crypto';

if (process.version !== 'v26.11.1') throw new Error('Node 26.11.1 required');
const files = ['launch', 'runner.mjs', 'package.json', 'package-lock.json', 'node/bin/node'];
async function collect(dir) {
  for (const name of (await readdir(dir)).sort()) {
    const path = `${dir}/${name}`;
    const stat = await lstat(path);
    if (stat.isDirectory()) await collect(path);
    else if (stat.isFile()) files.push(path);
    else throw new Error(`nonregular packaged asset: ${path}`);
  }
}
await collect('node_modules');
let identity = '';
for (const path of files) {
  identity += `${createHash('sha256').update(await readFile(path)).digest('hex')}  ${path}\n`;
}
await writeFile('identity.sha256', identity);
