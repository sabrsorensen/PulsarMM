export function createNexusAuthFeature(deps) {
  const {
    invoke,
    i18n,
    appState,
    nexusAuthBtn,
    nexusAccountStatus,
    setApiKey,
  } = deps;

  function setLoggedInUi(username) {
    appState.nexusUsername = username;
    nexusAccountStatus.textContent = i18n.get('statusConnectedAs', { name: username });
    nexusAccountStatus.classList.add('logged-in');
    nexusAuthBtn.textContent = i18n.get('disconnectBtn');
    nexusAuthBtn.className = 'modal-btn-delete';
    nexusAuthBtn.style.width = '100px';
    nexusAuthBtn.style.padding = '5px';
  }

  function setLoggedOutUi() {
    appState.nexusUsername = null;
    nexusAccountStatus.textContent = i18n.get('statusNotLoggedIn');
    nexusAccountStatus.classList.remove('logged-in');
    nexusAuthBtn.textContent = i18n.get('connectBtn');
    nexusAuthBtn.className = 'modal-btn-confirm';
    nexusAuthBtn.style.width = '100px';
    nexusAuthBtn.style.padding = '5px';
  }

  async function validateLoginState(preloadedKey = null) {
    try {
      const apiKey = preloadedKey || await invoke('get_nexus_api_key');
      setApiKey(apiKey);

      const response = await invoke('http_request', {
        url: 'https://api.nexusmods.com/v1/users/validate.json',
        method: 'GET',
        headers: { apikey: apiKey },
      });

      if (response.status < 200 || response.status >= 300) {
        throw new Error('Key validation failed with Nexus.');
      }

      const userData = JSON.parse(response.body);
      setLoggedInUi(userData.name);
      return true;
    } catch (e) {
      const errorStr = String(e);
      if (!errorStr.includes('No API Key found')) {
        console.warn('Login check:', e);
      }

      setLoggedOutUi();
      setApiKey('');
      return false;
    }
  }

  async function handleAuthButtonClick() {
    const isLoggedIn = nexusAccountStatus.classList.contains('logged-in');

    if (isLoggedIn) {
      const confirmed = await window.customConfirm(
        i18n.get('disconnectNexusAccMsg'),
        i18n.get('disconnectNexusAccTitle')
      );
      if (confirmed) {
        await invoke('logout_nexus');
        await validateLoginState();
      }
      return;
    }

    try {
      nexusAccountStatus.textContent = 'Waiting for browser...';
      nexusAuthBtn.disabled = true;
      const newKey = await invoke('login_to_nexus');

      if (newKey) {
        await window.customAlert('Successfully connected!', 'Success');
        await validateLoginState(newKey);
      }
    } catch (error) {
      await window.customAlert(`Login Failed: ${error}`, 'Error');
      await validateLoginState();
    } finally {
      nexusAuthBtn.disabled = false;
    }
  }

  return {
    validateLoginState,
    handleAuthButtonClick,
  };
}
