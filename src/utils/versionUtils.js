export function getBaseName(name) {
  if (!name) return '';
  let clean = name.toLowerCase();

  clean = clean.replace(/\.(zip|rar|7z|pak)$/i, '');
  clean = clean.replace(/-\d+(-\d+)*$/i, '');
  clean = clean.replace(/[- _]?v?\d+(\.\d+)*[a-z]?/gi, '');
  clean = clean.replace(/[^a-z]/g, '');

  return clean;
}

export function isNewerVersionAvailable(installedVersion, latestVersion) {
  if (!installedVersion || !latestVersion) {
    return false;
  }
  const regex = /^([0-9.]+)(.*)$/;
  const installedMatch = String(installedVersion).match(regex) || [];
  const latestMatch = String(latestVersion).match(regex) || [];
  const installedNumeric = (installedMatch[1] || '0').split('.').map(Number);
  const latestNumeric = (latestMatch[1] || '0').split('.').map(Number);
  const installedSuffix = installedMatch[2] || '';
  const latestSuffix = latestMatch[2] || '';
  const len = Math.max(installedNumeric.length, latestNumeric.length);
  for (let i = 0; i < len; i++) {
    const installedPart = installedNumeric[i] || 0;
    const latestPart = latestNumeric[i] || 0;
    if (latestPart > installedPart) return true;
    if (latestPart < installedPart) return false;
  }
  if (!installedSuffix && latestSuffix) return true;
  if (installedSuffix && !latestSuffix) return false;
  if (installedSuffix && latestSuffix) return latestSuffix > installedSuffix;
  return false;
}
