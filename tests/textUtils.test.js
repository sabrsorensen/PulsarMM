import test from 'node:test';
import assert from 'node:assert/strict';

import { bbcodeToHtml } from '../src/utils/textUtils.js';

function createDocumentStub() {
  return {
    createElement: (tag) => {
      if (tag !== 'textarea') {
        throw new Error(`Unsupported element: ${tag}`);
      }
      let html = '';
      return {
        set innerHTML(v) {
          html = String(v);
        },
        get value() {
          return html
            .replace(/&lt;/g, '<')
            .replace(/&gt;/g, '>')
            .replace(/&amp;/g, '&')
            .replace(/&quot;/g, '"')
            .replace(/&#39;/g, "'");
        },
      };
    },
  };
}

test('bbcodeToHtml handles simple inline tags', () => {
  assert.equal(
    bbcodeToHtml('[b]Bold[/b] [i]Italics[/i] [u]Underline[/u]'),
    '<p><strong>Bold</strong> <em>Italics</em> <u>Underline</u></p>'
  );
});

test('bbcodeToHtml handles url/list/quote and preserves code blocks escaped', () => {
  const prevDocument = global.document;
  global.document = createDocumentStub();

  try {
    const html = bbcodeToHtml(
      '[url=https://x.y]Link[/url]\n\n' +
      '[list][*]A[*]B[/list]\n\n' +
      '[quote]Q[/quote]\n\n' +
      '[code]&lt;tag&gt;1&lt;/tag&gt;<br>line2[/code]'
    );

    assert.match(html, /<a href="https:\/\/x\.y" target="_blank">Link<\/a>/);
    assert.match(html, /<ul><li>A<\/li><li>B<\/li><\/ul>/);
    assert.match(html, /<blockquote>Q<\/blockquote>/);
    assert.match(html, /<pre><code>&lt;tag&gt;1&lt;\/tag&gt;\nline2<\/code><\/pre>/);
  } finally {
    global.document = prevDocument;
  }
});

test('bbcodeToHtml falls back to inherit font-size for unknown size tokens', () => {
  const html = bbcodeToHtml('[size=99]Huge?[/size]');
  assert.equal(html, '<p><span style="font-size: inherit">Huge?</span></p>');
});

test('bbcodeToHtml handles additional tags and ordered list variant', () => {
  const html = bbcodeToHtml(
    '[center]C[/center]\n\n' +
    '[img]https://example.com/x.png[/img]\n\n' +
    '[color=red]R[/color]\n\n' +
    '[list=1][*]One[*]Two[/list]'
  );

  assert.match(html, /<div class="bbcode-center">C<\/div>/);
  assert.match(html, /<img class="bbcode-img" src="https:\/\/example\.com\/x\.png" \/>/);
  assert.match(html, /<span style="color: red">R<\/span>/);
  assert.match(html, /<ol><li>One<\/li><li>Two<\/li><\/ol>/);
});

test('bbcodeToHtml returns empty string for falsy input', () => {
  assert.equal(bbcodeToHtml(''), '');
  assert.equal(bbcodeToHtml(null), '');
});

test('bbcodeToHtml skips empty paragraphs created by extra blank lines', () => {
  const html = bbcodeToHtml('Line 1\n\n\n\nLine 2');
  assert.equal(html, '<p>Line 1</p><p>Line 2</p>');
});

test('bbcodeToHtml supports known size mapping and multiple list passes', () => {
  const html = bbcodeToHtml(
    '[size=3]Sized[/size]\n\n' +
    '[list][*]A[/list]\n\n' +
    '[list=1][*]B[/list]'
  );

  assert.match(html, /<span style="font-size: 14px">Sized<\/span>/);
  assert.match(html, /<ul><li>A<\/li><\/ul>/);
  assert.match(html, /<ol><li>B<\/li><\/ol>/);
});

test('bbcodeToHtml handles leading and trailing blank paragraphs', () => {
  const html = bbcodeToHtml('\n\nHead\n\nTail\n\n');
  assert.equal(html, '<p>Head</p><p>Tail</p>');
});
