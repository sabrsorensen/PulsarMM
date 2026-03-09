export function createLaunchGameHandler(deps) {
  const {
    appState,
    launchBtn,
    launchText,
    i18n,
    invoke,
    addAppLog,
  } = deps;

  return async function handleLaunchGameClick() {
    if (!appState.gamePath || !appState.versionType) return;
    if (launchBtn.classList.contains('is-launching')) return;

    const originalText = launchText.textContent;
    launchBtn.classList.add('is-launching');
    launchText.textContent = i18n.get('launchingStateText');

    try {
      const launchStrategy = await invoke('launch_game', {
        versionType: appState.versionType,
        gamePath: appState.gamePath,
      });
      addAppLog(`Game launch command sent via: ${launchStrategy}`, 'INFO');

      setTimeout(() => {
        launchBtn.classList.remove('is-launching');
        launchText.textContent = originalText;
      }, 10000);
    } catch (error) {
      launchBtn.classList.remove('is-launching');
      launchText.textContent = originalText;
      addAppLog(`Game launch failed: ${error}`, 'ERROR');
      await window.customAlert(`Failed to launch game: ${error}`, 'Launch Error');
    }
  };
}
