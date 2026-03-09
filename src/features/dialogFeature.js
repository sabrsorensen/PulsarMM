export function installDialogHelpers({ getText }) {
  const inputModal = document.getElementById('inputDialogModal');
  const inputTitle = document.getElementById('inputDialogTitle');
  const inputMessage = document.getElementById('inputDialogMessage');
  const inputField = document.getElementById('inputDialogField');
  const inputCancel = document.getElementById('inputDialogCancelBtn');
  const inputOk = document.getElementById('inputDialogOkBtn');

  const genericDialogModal = document.getElementById('genericDialogModal');
  const genericDialogTitle = document.getElementById('genericDialogTitle');
  const genericDialogMessage = document.getElementById('genericDialogMessage');
  const genericDialogActions = document.getElementById('genericDialogActions');

  window.customPrompt = (message, title, defaultValue = '') => new Promise((resolve) => {
    inputTitle.textContent = title || 'Pulsar';
    inputMessage.textContent = message;
    inputField.value = defaultValue;

    const cleanup = () => {
      inputModal.classList.add('hidden');
      inputOk.onclick = null;
      inputCancel.onclick = null;
      inputField.onkeydown = null;
    };

    const confirm = () => {
      const val = inputField.value;
      cleanup();
      resolve(val);
    };

    const cancel = () => {
      cleanup();
      resolve(null);
    };

    inputOk.onclick = confirm;
    inputCancel.onclick = cancel;
    inputField.onkeydown = (e) => {
      if (e.key === 'Enter') confirm();
      if (e.key === 'Escape') cancel();
    };

    inputModal.classList.remove('hidden');
    setTimeout(() => inputField.focus(), 50);
  });

  function showDialog(title, message, type = 'alert', confirmText = null, cancelText = null) {
    const finalConfirmText = confirmText || getText('okBtn');
    const finalCancelText = cancelText || getText('cancelBtn');

    return new Promise((resolve) => {
      genericDialogTitle.textContent = title || 'Pulsar';
      genericDialogMessage.textContent = message;
      genericDialogActions.innerHTML = '';

      if (type === 'confirm') {
        const btnCancel = document.createElement('button');
        btnCancel.className = 'modal-gen-btn-cancel';
        btnCancel.textContent = finalCancelText;
        if (finalCancelText.length > 8) {
          btnCancel.style.width = 'auto';
          btnCancel.style.paddingLeft = '15px';
          btnCancel.style.paddingRight = '15px';
        }
        btnCancel.onclick = () => {
          genericDialogModal.classList.add('hidden');
          resolve(false);
        };
        genericDialogActions.appendChild(btnCancel);
      }

      const btnOk = document.createElement('button');
      btnOk.className = 'modal-gen-btn-confirm';
      btnOk.textContent = finalConfirmText;
      btnOk.onclick = () => {
        genericDialogModal.classList.add('hidden');
        resolve(true);
      };
      genericDialogActions.appendChild(btnOk);
      genericDialogModal.classList.remove('hidden');
    });
  }

  window.customAlert = async (msg, title) => {
    await showDialog(title, msg, 'alert');
  };

  window.customConfirm = async (msg, title, confirmBtnText = null, cancelBtnText = null) =>
    showDialog(title, msg, 'confirm', confirmBtnText, cancelBtnText);

  window.customConflictDialog = (message, title, btnReplaceText, btnKeepText, btnCancelText) =>
    new Promise((resolve) => {
      genericDialogTitle.textContent = title || 'Conflict';
      genericDialogMessage.textContent = message;
      genericDialogActions.innerHTML = '';

      const btnCancel = document.createElement('button');
      btnCancel.className = 'modal-gen-btn-cancel';
      btnCancel.textContent = btnCancelText || 'Cancel';
      btnCancel.onclick = () => {
        genericDialogModal.classList.add('hidden');
        resolve('cancel');
      };
      genericDialogActions.appendChild(btnCancel);

      const btnKeep = document.createElement('button');
      btnKeep.className = 'modal-gen-btn-confirm';
      btnKeep.textContent = btnKeepText || 'Keep Both';
      btnKeep.onclick = () => {
        genericDialogModal.classList.add('hidden');
        resolve('keep');
      };
      genericDialogActions.appendChild(btnKeep);

      const btnReplace = document.createElement('button');
      btnReplace.className = 'modal-gen-btn-confirm';
      btnReplace.textContent = btnReplaceText || 'Replace';
      btnReplace.onclick = () => {
        genericDialogModal.classList.add('hidden');
        resolve('replace');
      };
      genericDialogActions.appendChild(btnReplace);

      genericDialogModal.classList.remove('hidden');
    });
}
