export const escapeXml = (unsafe) => unsafe.replace(/[<>"'&]/g, char => ({ '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;', '&': '&amp;' }[char]));

export function formatNode(node, indentLevel = 0) {
  const indent = '  '.repeat(indentLevel);
  let attributeList = Array.from(node.attributes);
  const nameAttr = node.getAttribute('name');
  const isMainDataContainer = (nameAttr === 'Data' && node.parentNode?.tagName === 'Data');
  const isDependenciesTag = (nameAttr === 'Dependencies');

  if (isMainDataContainer || isDependenciesTag) {
    attributeList = attributeList.filter(attr => attr.name !== 'value');
  }

  const attributes = attributeList.map(attr => `${attr.name}="${escapeXml(attr.value)}"`).join(' ');
  const tag = node.tagName;
  let nodeString = `${indent}<${tag}${attributes ? ' ' + attributes : ''}`;

  if (node.children.length > 0) {
    nodeString += '>\n';
    for (const child of node.children) {
      nodeString += formatNode(child, indentLevel + 1);
    }
    nodeString += `${indent}</${tag}>\n`;
  } else {
    nodeString += ' />\n';
  }

  return nodeString;
}
