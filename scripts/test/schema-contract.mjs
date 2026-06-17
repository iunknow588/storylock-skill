import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { readdirSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const requiredSchemas = [
  'src/storylock-local-story-access-skill/assets/schemas/access-response.schema.json',
  'src/storylock-local-story-access-skill/assets/schemas/grid-verification-input.schema.json',
  'src/storylock-local-story-access-skill/assets/schemas/local-authorization-input.schema.json',
  'src/storylock-local-story-access-skill/assets/schemas/object-strength-policy-input.schema.json',
  'src/storylock-local-story-processing-skill/assets/schemas/story-draft-input.schema.json',
  'src/storylock-local-story-processing-skill/assets/schemas/story-refine-input.schema.json',
  'src/storylock-remote-gateway-skill/assets/schemas/delegated-sign-input.schema.json',
  'src/storylock-remote-gateway-skill/assets/schemas/remote-gateway-request.schema.json',
  'src/storylock-remote-gateway-skill/assets/schemas/remote-gateway-response.schema.json',
];

function walk(dir, suffix, output = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(fullPath, suffix, output);
    } else if (entry.name.endsWith(suffix)) {
      output.push(fullPath);
    }
  }
  return output;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

const schemaPaths = walk(fileURLToPath(new URL('../../src/', import.meta.url)), '.schema.json');
assert.ok(schemaPaths.length > 0, 'no schema files found');

for (const required of requiredSchemas) {
  const schemaPath = new URL(`../../${required}`, import.meta.url);
  readJson(schemaPath);
}

for (const schemaPath of schemaPaths) {
  const schema = readJson(schemaPath);
  const displayPath = relative(root, schemaPath);
  assert.equal(schema.$schema, 'https://json-schema.org/draft/2020-12/schema', `${displayPath} must use draft 2020-12`);
  assert.equal(schema.type, 'object', `${displayPath} must describe an object`);
  assert.ok(schema.title, `${displayPath} must include title`);
}

const remoteRequest = readJson(new URL('../../src/storylock-remote-gateway-skill/assets/schemas/remote-gateway-request.schema.json', import.meta.url));
assert.deepEqual(
  remoteRequest.properties.capability.enum,
  ['requestSignature', 'requestPasswordFill'],
  'remote gateway request capabilities must match current mainline',
);

const remoteResponse = readJson(new URL('../../src/storylock-remote-gateway-skill/assets/schemas/remote-gateway-response.schema.json', import.meta.url));
assert.deepEqual(
  remoteResponse.properties.capability.enum,
  ['requestSignature', 'requestPasswordFill'],
  'remote gateway response capabilities must match current mainline',
);

const accessResponse = readJson(new URL('../../src/storylock-local-story-access-skill/assets/schemas/access-response.schema.json', import.meta.url));
assert.match(
  accessResponse.properties.error.anyOf[0].properties.code.pattern,
  /\^SLG-\[0-9\]\{3\}\$/,
  'access response error code must use SLG-xxx pattern',
);

console.log(JSON.stringify({
  status: 'passed',
  schemasChecked: schemaPaths.length,
  requiredSchemas: requiredSchemas.length,
}, null, 2));
