export function createNxmHandlerFeature(deps) {
  const {
    nxmHandlerBtn,
    i18n,
    invoke,
    updateNXMButtonState,
  } = deps;

  function setStatus(message, type = 'success') {
    const statusEl = document.getElementById('nxmHandlerStatus');
    statusEl.textContent = message;
    statusEl.className = `handler-status status-${type}`;
    statusEl.classList.remove('hidden');
  }

  async function handleNxmHandlerClick() {
    nxmHandlerBtn.disabled = true;
    const isCurrentlyRegistered = await invoke('is_protocol_handler_registered');

    try {
      if (isCurrentlyRegistered) {
        const confirmed = await window.customConfirm(
          i18n.get('removeNXMHandlerMsg'),
          i18n.get('removeNXMHandlerTitle')
        );
        if (!confirmed) return;

        await invoke('unregister_nxm_protocol');
        await updateNXMButtonState();
        setStatus(i18n.get('nxmRemovedSuccess') || 'Successfully removed.', 'success');
        return;
      }

      const confirmed = await window.customConfirm(
        i18n.get('addNXMHandlerMsg'),
        i18n.get('addNXMHandlerTitle')
      );
      if (!confirmed) return;

      await invoke('register_nxm_protocol');
      await updateNXMButtonState();
      setStatus(i18n.get('nxmSetSuccess') || 'Successfully set!', 'success');
    } catch (error) {
      setStatus(`Error: ${error}`, 'error');
    } finally {
      nxmHandlerBtn.disabled = false;
    }
  }

  return {
    handleNxmHandlerClick,
  };
}
