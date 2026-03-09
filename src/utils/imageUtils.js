export async function loadImageViaTauri(invoke, imgElement, url) {
  try {
    const response = await invoke('http_request', {
      url,
      method: 'GET',
      headers: {},
    });

    if (response.status >= 200 && response.status < 300) {
      const contentType = response.headers['content-type'] || 'image/jpeg';
      const dataURL = `data:${contentType};base64,${response.body}`;
      imgElement.src = dataURL;
    } else {
      throw new Error(`HTTP ${response.status}`);
    }
  } catch (error) {
    console.warn(`Failed to load image via Tauri: ${url}`, error);
    imgElement.src = '/src/assets/placeholder.png';
  }
}
