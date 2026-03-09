import { CURATED_LIST_URL, CACHE_DURATION_MS } from '../config/appConstants.js';

export function createCuratedDataFeature(deps) {
  const {
    appDataDir,
    join,
    readTextFile,
    writeTextFile,
    mkdir,
    invoke,
    setCuratedData,
  } = deps;

  async function loadCuratedListFromCache() {
    try {
      const dataDir = await appDataDir();
      const cacheFilePath = await join(dataDir, 'curated_list_cache.json');
      const content = await readTextFile(cacheFilePath);
      const cachedData = JSON.parse(content);
      console.log('Successfully loaded curated list from local cache.');
      return cachedData;
    } catch {
      console.log('No local cache found.');
      return null;
    }
  }

  async function saveCuratedListToCache(data, etag = null) {
    try {
      const dataToCache = {
        timestamp: Date.now(),
        etag,
        data,
      };

      const dataDir = await appDataDir();
      await mkdir(dataDir, { recursive: true });
      const cacheFilePath = await join(dataDir, 'curated_list_cache.json');
      await writeTextFile(cacheFilePath, JSON.stringify(dataToCache));
      console.log('Saved fresh curated list to local cache.');
    } catch (error) {
      console.error('Failed to save curated list to cache:', error);
    }
  }

  async function fetchCuratedData() {
    const cachedObj = await loadCuratedListFromCache();

    if (cachedObj) {
      const isStale = Date.now() - cachedObj.timestamp > CACHE_DURATION_MS;
      if (!isStale) {
        setCuratedData(cachedObj.data);
        return;
      }
    }

    try {
      console.log('Checking for curated list updates...');

      const headers = {};
      if (cachedObj?.etag) {
        headers['If-None-Match'] = cachedObj.etag;
      }

      const response = await invoke('http_request', {
        url: CURATED_LIST_URL,
        method: 'GET',
        headers,
      });

      if (response.status === 304 && cachedObj) {
        console.log("Remote list hasn't changed. Extending cache duration.");
        setCuratedData(cachedObj.data);
        await saveCuratedListToCache(cachedObj.data, cachedObj.etag);
        return;
      }

      if (response.status < 200 || response.status >= 300) {
        throw new Error(`Could not fetch remote curated list. Status: ${response.status} ${response.status_text}`);
      }

      const freshData = JSON.parse(response.body);
      const newEtag = response.headers['etag'];
      setCuratedData(freshData);
      console.log(`Successfully loaded ${freshData.length} mods from network.`);
      await saveCuratedListToCache(freshData, newEtag);
    } catch (error) {
      console.error('CRITICAL: Could not load curated mod data:', error);
      if (cachedObj) {
        console.warn('Using stale cache due to network error.');
        setCuratedData(cachedObj.data);
      } else {
        setCuratedData([]);
        if (typeof window.addAppLog === 'function') {
          window.addAppLog(
            'Failed to load mod data from server. Browse and update checks are unavailable until connectivity returns.',
            'WARN'
          );
        }
        if (typeof window.customAlert === 'function') {
          await window.customAlert(
            'Failed to load mod data from the server. Update checks and the browse tab will not work.',
            'Network Error'
          );
        } else {
          window.alert('Network Error\n\nFailed to load mod data from the server. Update checks and the browse tab will not work.');
        }
      }
    }
  }

  return {
    fetchCuratedData,
  };
}
