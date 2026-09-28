import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const referencePath = new URL('../reference/madrashamaneger V2.6.reference.html', import.meta.url);
const workingPath = new URL('../src/index.html', import.meta.url);
const referenceBytes = await readFile(referencePath);
const hash = createHash('sha256').update(referenceBytes).digest('hex');
assert.equal(hash, '901592016ae4011839a25e8e1a924a71d9b7bb79b227eff7bcdd2b7e3a8164f8', 'immutable reference hash changed');

const normalizeNewlines = value => value.replaceAll('\r\n', '\n');
function visualSource(value) {
  let text = normalizeNewlines(value)
    .replaceAll('./assets/styles/fonts.css', 'REFERENCE_FONT_CSS')
    .replaceAll('https://fonts.maateen.me/solaiman-lipi/font.css', 'REFERENCE_FONT_CSS')
    .replaceAll('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap', 'REFERENCE_FONT_CSS')
    .replaceAll("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='150' height='150' viewBox='0 0 150 150'%3E%3Crect width='150' height='150' fill='%23f3f4f6'/%3E%3Ccircle cx='75' cy='57' r='25' fill='%23cbd5e1'/%3E%3Cpath d='M30 140c3-31 20-47 45-47s42 16 45 47' fill='%23cbd5e1'/%3E%3C/svg%3E", 'REFERENCE_PLACEHOLDER')
    .replaceAll('https://via.placeholder.com/150', 'REFERENCE_PLACEHOLDER');
  // Runtime implementation scripts may change; static DOM and all CSS must remain byte-equivalent.
  return text.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, '').replace(/^[ \t]*\n/gm, '').trim();
}
const reference = referenceBytes.toString('utf8');
const working = await readFile(workingPath, 'utf8');
assert.equal(visualSource(working), visualSource(reference), 'static DOM/CSS differs from Reference UI');

function classTokens(value) {
  const tokens = [];
  for (const match of value.matchAll(/class=["'`]([^"'`]*)["'`]/g)) tokens.push(match[1].trim().replace(/\s+/g,' '));
  return tokens.sort();
}
assert.deepEqual(classTokens(working), classTokens(reference), 'existing class attribute inventory changed');
console.log('REFERENCE_OK: immutable hash, static DOM/CSS and class inventory verified');
