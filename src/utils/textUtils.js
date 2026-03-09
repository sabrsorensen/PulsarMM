export function bbcodeToHtml(str) {
  if (!str) return '';
  const codeBlocks = [];
  str = str.replace(/\[code\]([\s\S]*?)\[\/code\]/gis, (match, content) => {
    const placeholder = `{{CODE_BLOCK_${codeBlocks.length}}}`;
    const tempElem = document.createElement('textarea');
    tempElem.innerHTML = content;
    let decodedContent = tempElem.value;
    decodedContent = decodedContent.replace(/<br\s*\/?>/gi, '\n');
    codeBlocks.push(decodedContent);
    return placeholder;
  });
  str = str.replace(/<br\s*\/?>/gi, '\n');
  const listReplacer = (match, listTag, content) => {
    const listType = listTag.includes('1') ? 'ol' : 'ul';
    const items = content.split(/\[\*\]/g).slice(1).map(item => `<li>${item.trim()}</li>`).join('');
    return `<${listType}>${items}</${listType}>`;
  };
  while (/\[(list|list=1)\]([\s\S]*?)\[\/list\]/i.test(str)) {
    str = str.replace(/\[(list|list=1)\]([\s\S]*?)\[\/list\]/i, listReplacer);
  }
  str = str.replace(/\[quote\]([\s\S]*?)\[\/quote\]/gis, '<blockquote>$1</blockquote>');
  str = str.replace(/\[center\]([\s\S]*?)\[\/center\]/gis, '<div class="bbcode-center">$1</div>');
  str = str.replace(/\[img\](.*?)\[\/img\]/gis, '<img class="bbcode-img" src="$1" />');
  str = str.replace(/\[b\](.*?)\[\/b\]/gis, '<strong>$1</strong>');
  str = str.replace(/\[i\](.*?)\[\/i\]/gis, '<em>$1</em>');
  str = str.replace(/\[u\](.*?)\[\/u\]/gis, '<u>$1</u>');
  str = str.replace(/\[url=(.*?)\](.*?)\[\/url\]/gis, '<a href="$1" target="_blank">$2</a>');
  str = str.replace(/\[color=(.*?)\](.*?)\[\/color\]/gis, '<span style="color: $1">$2</span>');
  str = str.replace(/\[size=(\d+)\](.*?)\[\/size\]/gis, (match, size, text) => {
    const sizeMap = { '1': '10px', '2': '12px', '3': '14px', '4': '16px', '5': '18px' };
    const fontSize = sizeMap[size] || 'inherit';
    return `<span style="font-size: ${fontSize}">${text}</span>`;
  });
  const paragraphs = str.split(/\n\s*\n/);
  let html = paragraphs.map(p => {
    let para = p.trim();
    if (!para) return '';
    const isBlock = para.startsWith('<blockquote') || para.startsWith('<ol') || para.startsWith('<ul') || para.startsWith('{{CODE_BLOCK') || para.startsWith('<div class="bbcode-center') || para.startsWith('<img');
    para = para.replace(/\n/g, '<br>');
    return isBlock ? para : `<p>${para}</p>`;
  }).join('');
  html = html.replace(/{{CODE_BLOCK_(\d+)}}/g, (match, index) => {
    const content = codeBlocks[parseInt(index, 10)];
    const escapedContent = content.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return `<pre><code>${escapedContent}</code></pre>`;
  });
  return html;
}
