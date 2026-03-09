export function createNexusApi({ invoke, getApiKey, cache }) {
  async function fetchDownloadUrlFromNexus(modId, fileId, queryParams = '') {
    let url = `https://api.nexusmods.com/v1/games/nomanssky/mods/${modId}/files/${fileId}/download_link.json`;

    if (queryParams) {
      url += `?${queryParams}`;
    }

    const headers = { apikey: getApiKey() };
    try {
      const response = await invoke('http_request', {
        url,
        method: 'GET',
        headers,
      });
      if (response.status < 200 || response.status >= 300) {
        console.error(`API Error ${response.status}:`, response.body);
        return null;
      }
      const data = JSON.parse(response.body);
      return data[0]?.URI;
    } catch (error) {
      console.error(`Failed to get download URL for mod ${modId}:`, error);
      return null;
    }
  }

  async function fetchModFilesFromNexus(modId) {
    const modIdStr = String(modId);
    if (cache.has(modIdStr)) {
      return cache.get(modIdStr);
    }
    const url = `https://api.nexusmods.com/v1/games/nomanssky/mods/${modIdStr}/files.json`;
    const headers = { apikey: getApiKey() };
    try {
      const response = await invoke('http_request', {
        url,
        method: 'GET',
        headers,
      });
      if (response.status < 200 || response.status >= 300) return null;
      const data = JSON.parse(response.body);
      cache.set(modIdStr, data);
      return data;
    } catch (error) {
      return null;
    }
  }

  return {
    fetchDownloadUrlFromNexus,
    fetchModFilesFromNexus,
  };
}
