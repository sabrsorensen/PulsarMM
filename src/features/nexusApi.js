export function createNexusApi({ invoke, getApiKey, cache }) {
  const API_BASE = 'https://api.nexusmods.com/v1/games/nomanssky';

  async function makeNexusApiCall(endpoint, requiresAuth = true) {
    const headers = {
      "User-Agent": "PulsarMM/1.0",
      "Application-Name": "PulsarMM",
      "Application-Version": "1.0"
    };

    if (requiresAuth) {
      const apiKey = getApiKey();
      if (!apiKey) {
        throw new Error('NexusMods API key required. Please configure your API key in settings.');
      }
      headers.apikey = apiKey;
    }

    try {
      const response = await invoke('http_request', {
        url: `${API_BASE}${endpoint}`,
        method: 'GET',
        headers,
      });

      if (response.status === 401) {
        throw new Error('Invalid NexusMods API key. Please check your API key in settings.');
      }

      if (response.status === 429) {
        throw new Error('NexusMods API rate limit exceeded. Please try again later.');
      }

      if (response.status < 200 || response.status >= 300) {
        console.error(`API Error ${response.status}:`, response.body);
        throw new Error(`NexusMods API error: ${response.status} ${response.status_text}`);
      }

      return JSON.parse(response.body);
    } catch (error) {
      console.error(`NexusMods API call failed:`, error);
      throw error;
    }
  }

  async function fetchTrendingMods() {
    console.log('Fetching trending mods from NexusMods...');
    try {
      const data = await makeNexusApiCall('/mods/trending.json');
      console.log(`Successfully fetched ${data.length} trending mods`);
      return data;
    } catch (error) {
      console.error('Failed to fetch trending mods:', error);
      throw error;
    }
  }

  async function fetchRecentlyUpdatedMods(period = '1w') {
    console.log(`Fetching recently updated mods (${period}) from NexusMods...`);
    try {
      const data = await makeNexusApiCall(`/mods/updated.json?period=${period}`);
      console.log(`Successfully fetched ${data.length} recently updated mods`);
      return data;
    } catch (error) {
      console.error(`Failed to fetch recently updated mods (${period}):`, error);
      throw error;
    }
  }

  async function fetchModDetails(modId) {
    console.log(`Fetching details for mod ${modId}...`);
    const cacheKey = `mod_${modId}`;

    if (cache.has(cacheKey)) {
      console.log(`Using cached data for mod ${modId}`);
      return cache.get(cacheKey);
    }

    try {
      const data = await makeNexusApiCall(`/mods/${modId}.json`);
      console.log(`Successfully fetched details for mod ${modId}: "${data.name}"`);

      // Cache the mod details
      cache.set(cacheKey, data);
      return data;
    } catch (error) {
      console.error(`Failed to fetch mod details for ${modId}:`, error);
      throw error;
    }
  }

  async function fetchModFiles(modId) {
    console.log(`Fetching files for mod ${modId}...`);
    const cacheKey = `files_${modId}`;

    if (cache.has(cacheKey)) {
      console.log(`Using cached files data for mod ${modId}`);
      return cache.get(cacheKey);
    }

    try {
      const data = await makeNexusApiCall(`/mods/${modId}/files.json`);
      console.log(`Successfully fetched ${data.files?.length || 0} files for mod ${modId}`);

      // Cache the files data
      cache.set(cacheKey, data);
      return data;
    } catch (error) {
      console.error(`Failed to fetch files for mod ${modId}:`, error);
      throw error;
    }
  }

  async function fetchModChangelogs(modId) {
    console.log(`Fetching changelogs for mod ${modId}...`);
    try {
      const data = await makeNexusApiCall(`/mods/${modId}/changelogs.json`);
      console.log(`Successfully fetched changelogs for mod ${modId}`);
      return data;
    } catch (error) {
      console.error(`Failed to fetch changelogs for mod ${modId}:`, error);
      // Return empty object if changelogs fail - not critical
      return {};
    }
  }

  async function fetchCompleteModData(modId) {
    console.log(`Fetching complete data for mod ${modId}...`);
    try {
      // Fetch all data for a mod in parallel
      const [modDetails, modFiles, modChangelogs] = await Promise.all([
        fetchModDetails(modId),
        fetchModFiles(modId),
        fetchModChangelogs(modId)
      ]);

      // Combine into the format expected by the UI
      const completeModData = {
        mod_id: modDetails.mod_id,
        name: modDetails.name,
        summary: modDetails.summary,
        version: modDetails.version,
        picture_url: modDetails.picture_url,
        author: modDetails.author,
        updated_timestamp: modDetails.updated_timestamp,
        created_timestamp: modDetails.created_timestamp,
        description: modDetails.description,
        unique_downloads: modDetails.unique_downloads,
        endorsement_count: modDetails.endorsement_count,
        status: modDetails.status,
        contains_adult_content: modDetails.contains_adult_content,
        // Add files and changelogs
        files: modFiles.files || [],
        changelogs: modChangelogs || {},
        // Add default warning state (can be customized later)
        state: 'normal',
        warningMessage: ''
      };

      console.log(`Successfully compiled complete data for mod ${modId}: "${completeModData.name}"`);
      return completeModData;
    } catch (error) {
      console.error(`Failed to fetch complete mod data for ${modId}:`, error);
      throw error;
    }
  }

  // Legacy download functions (keep for compatibility)
  async function fetchDownloadUrlFromNexus(modId, fileId, queryParams = '') {
    let url = `/mods/${modId}/files/${fileId}/download_link.json`;

    if (queryParams) {
      url += `?${queryParams}`;
    }

    try {
      const data = await makeNexusApiCall(url);
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

    try {
      const data = await makeNexusApiCall(`/mods/${modIdStr}/files.json`);
      cache.set(modIdStr, data);
      return data;
    } catch (error) {
      return null;
    }
  }

  return {
    // New direct API functions
    fetchTrendingMods,
    fetchRecentlyUpdatedMods,
    fetchModDetails,
    fetchModFiles,
    fetchModChangelogs,
    fetchCompleteModData,

    // Legacy functions (keep for compatibility)
    fetchDownloadUrlFromNexus,
    fetchModFilesFromNexus,
  };
}
