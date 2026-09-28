import { readFile, access } from 'node:fs/promises';
import assert from 'node:assert/strict';

const html = await readFile(new URL('../src/index.html', import.meta.url), 'utf8');
const forbidden = [
  'https://cdn.tailwindcss.com', 'https://fonts.maateen.me', 'https://fonts.googleapis.com',
  'https://fonts.gstatic.com', 'https://unpkg.com/lucide', 'https://html2canvas.hertzen.com',
  'https://cdn.jsdelivr.net/npm/sortablejs', 'https://via.placeholder.com'
];
for (const url of forbidden) assert.equal(html.includes(url), false, `remote runtime asset remains: ${url}`);
const required = [
  '../src/assets/vendor/tailwindcss-3.4.17.js', '../src/assets/vendor/lucide-0.468.0.min.js',
  '../src/assets/vendor/html2canvas-1.4.1.min.js', '../src/assets/vendor/sortable-1.15.0.min.js',
  '../src/assets/styles/fonts.css', '../src/assets/fonts/solaimanlipi-normal-v1.0.woff2', '../src/assets/fonts/inter-400.ttf'
];
for (const path of required) await access(new URL(path, import.meta.url));
console.log('OFFLINE_ASSETS_OK: all mandatory UI dependencies are local');
