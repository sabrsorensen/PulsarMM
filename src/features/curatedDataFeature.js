import { CACHE_DURATION_MS } from '../config/appConstants.js';

export function createCuratedDataFeature(deps) {
  const {
    appDataDir,
    join,
    readTextFile,
    writeTextFile,
    mkdir,
    invoke,
    setCuratedData,
    nexusApi,
    getApiKey,
  } = deps;

  async function loadModListFromCache() {
    try {
      const dataDir = await appDataDir();
      const cacheFilePath = await join(dataDir, 'mod_list_cache.json');
      const content = await readTextFile(cacheFilePath);
      const cachedData = JSON.parse(content);
      console.log('Successfully loaded mod list from local cache.');
      return cachedData;
    } catch {
      console.log('No local cache found.');
      return null;
    }
  }

  async function saveModListToCache(data) {
    try {
      const dataToCache = {
        timestamp: Date.now(),
        data,
      };

      const dataDir = await appDataDir();
      await mkdir(dataDir, { recursive: true });
      const cacheFilePath = await join(dataDir, 'mod_list_cache.json');
      await writeTextFile(cacheFilePath, JSON.stringify(dataToCache));
      console.log('Saved fresh mod list to local cache.');
    } catch (error) {
      console.error('Failed to save mod list to cache:', error);
    }
  }

  async function fetchModsFromNexusApi() {
    console.log('Fetching mods directly from NexusMods API...');

    try {
      if (!getApiKey()) {
        throw new Error('NexusMods API key required. Please configure your API key in settings to browse mods.');
      }

      // Fetch trending and recent mods in parallel
      const [trendingMods, recentMods] = await Promise.all([
        nexusApi.fetchTrendingMods().catch(err => {
          console.warn('Failed to fetch trending mods:', err);
          return [];
        }),
        nexusApi.fetchRecentlyUpdatedMods('1w').catch(err => {
          console.warn('Failed to fetch recent mods:', err);
          return [];
        })
      ]);

      console.log(`Fetched ${trendingMods.length} trending and ${recentMods.length} recent mods`);

      // Combine and deduplicate mods by mod_id
      const allModsMap = new Map();

      // Add trending mods (prioritize these)
      for (const mod of trendingMods) {
        if (mod.mod_id) {
          allModsMap.set(mod.mod_id, mod);
        }
      }

      // Add recent mods (don't overwrite trending)
      for (const mod of recentMods) {
        if (mod.mod_id && !allModsMap.has(mod.mod_id)) {
          allModsMap.set(mod.mod_id, mod);
        }
      }

      const uniqueMods = Array.from(allModsMap.values());
      console.log(`Combined into ${uniqueMods.length} unique mods`);

      // For each mod, ensure we have complete data
      const completeModsPromises = uniqueMods.slice(0, 50).map(async (basicModData) => {
        try {
          // Check if this is already complete mod data (from trending endpoint)
          if (basicModData.files && basicModData.description) {
            console.log(`Using complete data for mod ${basicModData.mod_id}: "${basicModData.name}"`);
            return {
              ...basicModData,
              state: 'normal', // Add default warning state
              warningMessage: ''
            };
          }

          // Fetch complete data for this mod
          return await nexusApi.fetchCompleteModData(basicModData.mod_id);
        } catch (error) {
          console.warn(`Failed to get complete data for mod ${basicModData.mod_id}:`, error);
          // Return basic data with defaults if complete fetch fails
          return {
            ...basicModData,
            files: [],
            changelogs: {},
            state: 'normal',
            warningMessage: '',
            description: basicModData.summary || 'No description available.'
          };
        }
      });

      // Process mods in smaller batches to avoid overwhelming the API
      const batchSize = 10;
      const completeMods = [];

      for (let i = 0; i < completeModsPromises.length; i += batchSize) {
        const batchPromises = completeModsPromises.slice(i, i + batchSize);
        console.log(`Processing mod batch ${Math.floor(i/batchSize) + 1}/${Math.ceil(completeModsPromises.length/batchSize)}...`);

        const batchResults = await Promise.all(batchPromises);
        completeMods.push(...batchResults);

        // Add delay between batches to respect API limits
        if (i + batchSize < completeModsPromises.length) {
          await new Promise(resolve => setTimeout(resolve, 1000));
        }
      }

      console.log(`Successfully processed ${completeMods.length} mods with complete data`);
      return completeMods;

    } catch (error) {
      console.error('Failed to fetch mods from NexusMods API:', error);
      throw error;
    }
  }

  async function fetchCuratedData() {
    const cachedObj = await loadModListFromCache();

    // Check if cached data is still fresh
    if (cachedObj) {
      const isStale = Date.now() - cachedObj.timestamp > CACHE_DURATION_MS;
      if (!isStale) {
        console.log('Using fresh cached mod data');
        setCuratedData(cachedObj.data);
        return;
      }
    }

    try {
      console.log('Fetching fresh mod data from NexusMods API...');

      const freshData = await fetchModsFromNexusApi();

      setCuratedData(freshData);
      console.log(`Successfully loaded ${freshData.length} mods from NexusMods API.`);
      await saveModListToCache(freshData);

    } catch (error) {
      console.error('CRITICAL: Could not load mod data from NexusMods API:', error);

      if (cachedObj) {
        console.warn('Using stale cache due to API error.');
        setCuratedData(cachedObj.data);
      } else {
        setCuratedData([]);

        // Show appropriate error message based on error type
        let errorMessage = 'Failed to load mod data from NexusMods API.';
        let errorTitle = 'API Error';

        if (error.message.includes('API key required')) {
          errorMessage = 'NexusMods API key required to browse mods. Please configure your API key in Settings > NexusMods Integration.';
          errorTitle = 'API Key Required';
        } else if (error.message.includes('Invalid API key')) {
          errorMessage = 'Invalid NexusMods API key. Please check your API key in Settings > NexusMods Integration.';
          errorTitle = 'Invalid API Key';
        } else if (error.message.includes('rate limit')) {
          errorMessage = 'NexusMods API rate limit exceeded. Please try again later.';
          errorTitle = 'Rate Limited';
        }

        if (typeof window.addAppLog === 'function') {
          window.addAppLog(errorMessage, 'WARN');
        }

        if (typeof window.customAlert === 'function') {
          await window.customAlert(errorMessage, errorTitle);
        } else {
          window.alert(`${errorTitle}\n\n${errorMessage}`);
        }
      }
    }
  }

  // New function to search for specific mods
  async function searchMods(query) {
    console.log(`Searching for mods: "${query}"`);

    if (!getApiKey()) {
      throw new Error('NexusMods API key required for search. Please configure your API key in settings.');
    }

    // For now, search within existing cached data
    // In the future, this could be enhanced with actual search API endpoints
    const currentData = await loadModListFromCache();

    if (!currentData || !currentData.data) {
      await fetchCuratedData(); // Fetch fresh data if no cache
      return [];
    }

    const searchTerm = query.toLowerCase();
    const results = currentData.data.filter(mod => {
      if (!mod) return false;
      const name = (mod.name || '').toLowerCase();
      const author = (mod.author || '').toLowerCase();
      const summary = (mod.summary || '').toLowerCase();
      const description = (mod.description || '').toLowerCase();

      return name.includes(searchTerm) ||
             author.includes(searchTerm) ||
             summary.includes(searchTerm) ||
             description.includes(searchTerm);
    });

    console.log(`Search for "${query}" returned ${results.length} results`);
    return results;
  }

  return {
    fetchCuratedData,
    searchMods,
  };
}
