export function resolveUpdateCandidate(modFolderName, cachedModData, curatedData, helpers) {
  const { getBaseName, isNewerVersionAvailable } = helpers;

  const localModInfo = cachedModData?.local_info;
  if (!localModInfo) return null;

  const modId = localModInfo.mod_id;
  const installedVersion = localModInfo.version;
  const installedFileId = localModInfo.file_id;
  if (!modId || !installedVersion) return null;

  const remoteModInfo = curatedData.find(mod => String(mod.mod_id) === String(modId));
  if (!remoteModInfo || !Array.isArray(remoteModInfo.files)) return null;

  let installedBaseName = '';
  const installedFileOnNexus = remoteModInfo.files.find(f => String(f.file_id) === String(installedFileId));

  if (installedFileOnNexus) {
    installedBaseName = getBaseName(installedFileOnNexus.name || installedFileOnNexus.file_name);
  } else if (localModInfo.install_source) {
    installedBaseName = getBaseName(localModInfo.install_source);
  } else {
    installedBaseName = getBaseName(modFolderName);
  }

  if (!installedBaseName) return null;

  const candidateFiles = remoteModInfo.files.filter(f => {
    if (f.category_name === 'OLD_VERSION') return false;
    return getBaseName(f.name || f.file_name) === installedBaseName;
  });
  if (candidateFiles.length === 0) return null;

  const latestFile = [...candidateFiles].sort((a, b) => b.uploaded_timestamp - a.uploaded_timestamp)[0];
  if (!isNewerVersionAvailable(installedVersion, latestFile.version)) return null;

  return {
    modId: String(modId),
    installedVersion: String(installedVersion),
    installedFileId: installedFileId ? String(installedFileId) : '',
    latestVersion: String(latestFile.version || ''),
    latestFileId: String(latestFile.file_id),
    latestFileName: latestFile.file_name || latestFile.name || '',
    latestDisplayName: latestFile.name || latestFile.file_name || (remoteModInfo.name || modFolderName),
    modName: remoteModInfo.name || modFolderName,
    nexusUrl: `https://www.nexusmods.com/nomanssky/mods/${remoteModInfo.mod_id}`,
  };
}
