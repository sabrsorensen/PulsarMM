export function createFolderSelectionModal(deps) {
  const {
    invoke,
    folderSelectionModal,
    folderSelectionList,
    fsmCancelBtn,
    fsmInstallAllBtn,
    fsmInstallSelectedBtn,
    flattenStructureCb,
  } = deps;

  function getCleanZipName(rawName) {
    let clean = rawName.replace(/\.(zip|rar|7z)$/i, '');
    clean = clean.replace(/-[\d.]+(-[\d.]+)*$/, '');
    return clean;
  }

  function getCommonPrefix(paths) {
    if (!paths || paths.length === 0) return null;
    const separator = paths[0].includes('/') ? '/' : '\\';
    const firstPathParts = paths[0].split(separator);
    if (firstPathParts.length < 2) return null;
    const potentialParent = firstPathParts[0];
    const allShare = paths.every(p => p.startsWith(potentialParent + separator));
    return allShare ? potentialParent : null;
  }

  return function openFolderSelectionModal(folders, modName, tempId) {
    return new Promise((resolve) => {
      folderSelectionList.innerHTML = '';
      flattenStructureCb.checked = false;

      let rootLabel = getCleanZipName(modName);
      const commonParent = getCommonPrefix(folders);
      if (commonParent) rootLabel = commonParent;

      function updateParentCheckbox(element) {
        const parentContainer = element.closest('.fs-children');
        if (!parentContainer) return;
        const parentWrapper = parentContainer.closest('.fs-wrapper');
        if (!parentWrapper) return;
        const parentCheckbox = parentWrapper.querySelector('.fs-item-row > .folder-select-checkbox');
        if (!parentCheckbox) return;

        const siblings = parentContainer.querySelectorAll(':scope > .fs-wrapper > .fs-item-row > .folder-select-checkbox');
        const allChecked = Array.from(siblings).every(cb => cb.checked);
        const someChecked = Array.from(siblings).some(cb => cb.checked);

        if (allChecked) {
          parentCheckbox.checked = true;
          parentCheckbox.indeterminate = false;
        } else if (someChecked) {
          parentCheckbox.checked = false;
          parentCheckbox.indeterminate = true;
        } else {
          parentCheckbox.checked = false;
          parentCheckbox.indeterminate = false;
        }

        updateParentCheckbox(parentWrapper);
      }

      function handleCheckboxChange(targetCheckbox, wrapper) {
        const isChecked = targetCheckbox.checked;
        const childrenContainer = wrapper.querySelector('.fs-children');
        if (childrenContainer) {
          const childCheckboxes = childrenContainer.querySelectorAll('.folder-select-checkbox');
          childCheckboxes.forEach(cb => cb.checked = isChecked);
        }
        updateParentCheckbox(wrapper);
      }

      function renderTreeItem(name, relativePath, isDir, container, parentIsChecked = false, isPreloaded = false) {
        const wrapper = document.createElement('div');
        wrapper.className = 'fs-wrapper';

        const row = document.createElement('div');
        row.className = 'fs-item-row';
        row.title = relativePath;

        const expander = document.createElement('div');
        expander.className = isDir ? 'fs-expander' : 'fs-expander placeholder';
        expander.textContent = isDir ? '▶' : '';
        row.appendChild(expander);

        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.className = 'folder-select-checkbox fs-checkbox';
        checkbox.value = relativePath;
        if (parentIsChecked) checkbox.checked = true;
        checkbox.onclick = (e) => {
          e.stopPropagation();
          handleCheckboxChange(checkbox, wrapper);
        };
        row.appendChild(checkbox);

        const label = document.createElement('span');
        label.className = 'fs-label';
        const displayName = name.split(/[/\\]/).pop();
        label.textContent = displayName;
        label.style.color = isDir ? 'var(--c-text-primary)' : 'rgba(255,255,255,0.6)';
        if (!isDir) label.style.fontStyle = 'italic';
        row.appendChild(label);

        wrapper.appendChild(row);
        const childrenContainer = document.createElement('div');
        childrenContainer.className = 'fs-children hidden';
        wrapper.appendChild(childrenContainer);

        label.onclick = () => {
          checkbox.checked = !checkbox.checked;
          handleCheckboxChange(checkbox, wrapper);
        };

        if (isDir) {
          let loaded = isPreloaded;
          expander.onclick = async (e) => {
            e.stopPropagation();
            const isClosed = childrenContainer.classList.contains('hidden');
            if (isClosed) {
              expander.classList.add('open');
              childrenContainer.classList.remove('hidden');
              if (!loaded) {
                childrenContainer.innerHTML = '<div style="padding:5px 0 5px 25px; opacity:0.5; font-size:12px;">Loading...</div>';
                try {
                  const contents = await invoke('get_staging_contents', { tempId, relativePath });
                  childrenContainer.innerHTML = '';
                  if (contents.length === 0) {
                    childrenContainer.innerHTML = '<div style="padding:5px 0 5px 25px; opacity:0.5; font-size:12px;">(Empty)</div>';
                  } else {
                    contents.forEach(node => {
                      const childPath = relativePath === '.' ? node.name : `${relativePath}/${node.name}`;
                      renderTreeItem(node.name, childPath, node.is_dir, childrenContainer, checkbox.checked, false);
                    });
                  }
                  loaded = true;
                } catch (err) {
                  childrenContainer.innerHTML = `<div style="color:red; padding-left:25px; font-size:12px;">Error: ${err}</div>`;
                }
              }
            } else {
              expander.classList.remove('open');
              childrenContainer.classList.add('hidden');
            }
          };
        }
        container.appendChild(wrapper);
      }

      renderTreeItem(rootLabel, '.', true, folderSelectionList, false, true);
      const rootWrapper = folderSelectionList.firstElementChild;
      const rootExpander = rootWrapper.querySelector('.fs-expander');
      const rootChildren = rootWrapper.querySelector('.fs-children');

      folders.forEach(childName => {
        renderTreeItem(childName, childName, true, rootChildren, false, false);
      });

      if (rootExpander && rootChildren) {
        rootExpander.classList.add('open');
        rootChildren.classList.remove('hidden');
      }

      folderSelectionModal.classList.remove('hidden');

      const cleanup = () => {
        folderSelectionModal.classList.add('hidden');
        fsmCancelBtn.onclick = null;
        fsmInstallAllBtn.onclick = null;
        fsmInstallSelectedBtn.onclick = null;
      };

      fsmCancelBtn.onclick = () => {
        cleanup();
        resolve(null);
      };

      fsmInstallAllBtn.onclick = () => {
        const isFlatten = flattenStructureCb.checked;
        cleanup();
        resolve({ selected: [], flatten: isFlatten });
      };

      fsmInstallSelectedBtn.onclick = () => {
        let rawSelected = Array.from(document.querySelectorAll('.folder-select-checkbox:checked'))
          .map(cb => cb.value)
          .filter(val => val !== '.');

        rawSelected.sort();
        const finalSelected = [];
        for (const path of rawSelected) {
          const isRedundant = finalSelected.some(parent => path.startsWith(parent + '/') || path.startsWith(parent + '\\'));
          if (!isRedundant) finalSelected.push(path);
        }

        const isFlatten = flattenStructureCb.checked;
        cleanup();

        if (finalSelected.length === 0) resolve(null);
        else resolve({ selected: finalSelected, flatten: isFlatten });
      };
    });
  };
}
