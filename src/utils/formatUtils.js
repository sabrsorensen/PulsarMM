export function formatBytes(bytes, decimals = 1) {
  if (!+bytes) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function formatDate(timestamp) {
  if (!timestamp) return '...';
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString();
}

export function formatNexusDate(timestamp, lang) {
  if (!timestamp) return '---';
  const date = new Date(timestamp * 1000);
  const options = { year: 'numeric', month: '2-digit', day: '2-digit' };
  return date.toLocaleDateString(lang, options);
}

export function mapLangCode(langCode) {
  const map = { cn: 'zh-CN', kr: 'ko-KR' };
  return map[langCode] || langCode;
}
