import test from 'node:test';
import assert from 'node:assert/strict';

import { escapeXml, formatNode } from '../src/utils/xmlUtils.js';

function makeNode({ tagName = 'Property', attrs = {}, children = [], parentNode = null } = {}) {
  const attributes = Object.entries(attrs).map(([name, value]) => ({ name, value }));
  return {
    tagName,
    parentNode,
    children,
    attributes,
    getAttribute: (name) => attrs[name],
  };
}

test('escapeXml escapes all reserved characters', () => {
  const raw = `<"'\u0026>`;
  assert.equal(escapeXml(raw), '&lt;&quot;&apos;&amp;&gt;');
});

test('formatNode formats self-closing nodes with attributes', () => {
  const node = makeNode({ attrs: { name: 'A', value: 'B&C' } });
  const out = formatNode(node, 0);
  assert.equal(out, '<Property name="A" value="B&amp;C" />\n');
});

test('formatNode strips value attribute for main Data and Dependencies containers', () => {
  const parent = makeNode({ tagName: 'Data', attrs: {} });
  const mainData = makeNode({ attrs: { name: 'Data', value: 'GcSomething' }, parentNode: parent });
  const dep = makeNode({ attrs: { name: 'Dependencies', value: 'x' }, parentNode: parent });

  assert.equal(formatNode(mainData), '<Property name="Data" />\n');
  assert.equal(formatNode(dep), '<Property name="Dependencies" />\n');
});

test('formatNode renders children recursively with indentation', () => {
  const child = makeNode({ attrs: { name: 'Child', value: '1' } });
  const parent = makeNode({ attrs: { name: 'Root', value: '2' }, children: [child] });

  const out = formatNode(parent, 0);
  assert.equal(
    out,
    '<Property name="Root" value="2">\n' +
      '  <Property name="Child" value="1" />\n' +
      '</Property>\n'
  );
});

test('formatNode handles nodes with no attributes', () => {
  const node = makeNode({ attrs: {} });
  const out = formatNode(node, 0);
  assert.equal(out, '<Property />\n');
});
