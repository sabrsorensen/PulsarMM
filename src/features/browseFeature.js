export function createBrowseFeature(deps) {
  const {
    appState,
    getCuratedData,
    i18n,
    invoke,
    loadImageViaTauri,
    mapLangCode,
    formatNexusDate,
    bbcodeToHtml,
    getBaseName,
    isNewerVersionAvailable,
    appWindow,
    LogicalSize,
    PANEL_OPEN_WIDTH,
    getIsPanelOpen,
    setIsPanelOpen,
    elements,
  } = deps;

  const {
    browseGridContainer,
    browseSearchInput,
    browseSortSelect,
    browseFilterSelect,
    languageSelector,
    modDetailPanel,
    modDetailName,
    modDetailAuthor,
    modDetailImage,
    modDetailVersion,
    modDetailUpdated,
    modDetailDescription,
    modDetailInstallBtnContainer,
    modDetailSecondaryActions,
    modDetailInstalled,
    modDetailCreated,
    fileSelectionModalOverlay,
    fileSelectionModalTitle,
    fileSelectionListContainer,
    changelogModalOverlay,
    changelogModalTitle,
    changelogListContainer,
    paginationContainer,
  } = elements;

  function displayChangelogs(modName, changelogs) {
    changelogModalTitle.textContent = `Changelogs: ${modName}`;
    changelogListContainer.innerHTML = '';

    if (!changelogs || Object.keys(changelogs).length === 0) {
      changelogListContainer.innerHTML = '<p>No changelogs available for this mod.</p>';
      changelogModalOverlay.classList.remove('hidden');
      return;
    }

    const sortedVersions = Object.keys(changelogs).sort((a, b) => b.localeCompare(a, undefined, { numeric: true }));

    for (const version of sortedVersions) {
      const changes = changelogs[version];
      const versionTitle = document.createElement('h3');
      versionTitle.className = 'changelog-version';
      versionTitle.textContent = `Version ${version}`;
      changelogListContainer.appendChild(versionTitle);
      const list = document.createElement('ul');
      list.className = 'changelog-list';
      for (const change of changes) {
        const listItem = document.createElement('li');
        listItem.textContent = change;
        list.appendChild(listItem);
      }
      changelogListContainer.appendChild(list);
    }
    changelogModalOverlay.classList.remove('hidden');
  }

  function fetchAndRenderBrowseGrid() {
    const curatedData = getCuratedData();
    if (curatedData.length === 0) {
      browseGridContainer.innerHTML = '<h2>Curated list could not be loaded.</h2>';
      return;
    }
    filterAndDisplayMods();
  }

  function filterAndDisplayMods() {
    if (!browseSearchInput || !browseFilterSelect || !browseSortSelect) return;

    const searchTerm = browseSearchInput.value.toLowerCase().trim();
    const filterBy = browseFilterSelect.value;
    const sortBy = browseSortSelect.value;

    const curatedData = getCuratedData();
    if (!curatedData) return;

    let processedMods = [...curatedData];

    if (searchTerm) {
      processedMods = processedMods.filter(modData => {
        if (!modData) return false;

        const name = (modData.name || '').toLowerCase();
        const author = (modData.author || '').toLowerCase();
        const summary = (modData.summary || '').toLowerCase();

        return (
          name.includes(searchTerm) ||
          author.includes(searchTerm) ||
          summary.includes(searchTerm)
        );
      });
    }

    if (filterBy === 'installed') {
      processedMods = processedMods.filter(mod => mod && appState.installedModsMap.has(String(mod.mod_id)));
    } else if (filterBy === 'uninstalled') {
      processedMods = processedMods.filter(mod => mod && !appState.installedModsMap.has(String(mod.mod_id)));
    }

    if (sortBy === 'name_asc') {
      processedMods.sort((a, b) => {
        const nameA = a?.name || '';
        const nameB = b?.name || '';
        return nameA.localeCompare(nameB, undefined, { sensitivity: 'base' });
      });
    } else {
      processedMods.sort((a, b) => (b?.updated_timestamp || 0) - (a?.updated_timestamp || 0));
    }

    const totalItems = processedMods.length;
    const totalPages = Math.ceil(totalItems / appState.modsPerPage) || 1;

    if (appState.currentPage > totalPages) appState.currentPage = 1;
    if (appState.currentPage < 1) appState.currentPage = 1;

    const startIndex = (appState.currentPage - 1) * appState.modsPerPage;
    const endIndex = startIndex + appState.modsPerPage;

    const modsForCurrentPage = processedMods.slice(startIndex, endIndex);

    displayMods(modsForCurrentPage);
    renderPaginationControls(totalPages, appState.currentPage);
  }

  function renderPaginationControls(totalPages, currentPage) {
    const curatedData = getCuratedData();
    paginationContainer.innerHTML = '';

    const countDiv = document.createElement('div');
    countDiv.className = 'pagination-count';
    countDiv.textContent = i18n.get('browseTotalMods', { count: curatedData.length });
    paginationContainer.appendChild(countDiv);

    paginationContainer.classList.remove('hidden');

    if (totalPages <= 1) {
      return;
    }

    const createButton = (text, page, isActive = false, isDisabled = false) => {
      const btn = document.createElement('div');
      btn.className = `page-btn ${isActive ? 'active' : ''} ${isDisabled ? 'disabled' : ''}`;
      btn.textContent = text;
      if (!isDisabled && !isActive) {
        btn.onclick = () => {
          appState.currentPage = page;
          filterAndDisplayMods();
          browseGridContainer.scrollTop = 0;
        };
      }
      return btn;
    };

    const createDots = () => {
      const dots = document.createElement('span');
      dots.className = 'page-dots';
      dots.textContent = '...';
      return dots;
    };

    paginationContainer.appendChild(createButton('<', currentPage - 1, false, currentPage === 1));

    const MAX_VISIBLE_PAGES = 7;

    if (totalPages <= MAX_VISIBLE_PAGES) {
      for (let i = 1; i <= totalPages; i++) {
        paginationContainer.appendChild(createButton(i, i, i === currentPage));
      }
    } else if (currentPage <= 4) {
      for (let i = 1; i <= 5; i++) {
        paginationContainer.appendChild(createButton(i, i, i === currentPage));
      }
      paginationContainer.appendChild(createDots());
      paginationContainer.appendChild(createButton(totalPages, totalPages, totalPages === currentPage));
    } else if (currentPage >= totalPages - 3) {
      paginationContainer.appendChild(createButton(1, 1, 1 === currentPage));
      paginationContainer.appendChild(createDots());
      for (let i = totalPages - 4; i <= totalPages; i++) {
        paginationContainer.appendChild(createButton(i, i, i === currentPage));
      }
    } else {
      paginationContainer.appendChild(createButton(1, 1, 1 === currentPage));
      paginationContainer.appendChild(createDots());
      paginationContainer.appendChild(createButton(currentPage - 1, currentPage - 1));
      paginationContainer.appendChild(createButton(currentPage, currentPage, true));
      paginationContainer.appendChild(createButton(currentPage + 1, currentPage + 1));
      paginationContainer.appendChild(createDots());
      paginationContainer.appendChild(createButton(totalPages, totalPages, totalPages === currentPage));
    }

    paginationContainer.appendChild(createButton('>', currentPage + 1, false, currentPage === totalPages));
  }

  function displayMods(modsToDisplay) {
    browseGridContainer.innerHTML = '';
    const template = document.getElementById('modCardTemplate');
    if (modsToDisplay.length === 0) {
      browseGridContainer.innerHTML = '<h2>No mods match your search.</h2>';
      return;
    }

    for (const modData of modsToDisplay) {
      if (!modData) continue;
      const card = template.content.cloneNode(true).firstElementChild;
      card.dataset.modId = modData.mod_id;
      const modIdStr = String(modData.mod_id);
      if (appState.installedModsMap.has(modIdStr)) {
        card.classList.add('is-installed');
        card.querySelector('.mod-card-installed-badge').classList.remove('hidden');
      }

      const titleElement = card.querySelector('.mod-card-title');
      const thumbnailImg = card.querySelector('.mod-card-thumbnail');
      const imageUrl = modData.picture_url || '/src/assets/placeholder.png';

      if (imageUrl && imageUrl.startsWith('http')) {
        loadImageViaTauri(invoke, thumbnailImg, imageUrl);
      } else {
        thumbnailImg.src = imageUrl;
      }

      titleElement.title = modData.name;

      const versionSpan = `<span class="mod-card-version-inline">${modData.version || ''}</span>`;
      let titleHtml = modData.name + versionSpan;

      if (modData.state === 'warning') {
        card.classList.add('has-warning');
        if (modData.warningMessage) {
          const warningIconHtml = `<span class="warning-icon" title="${modData.warningMessage}">⚠️ </span>`;
          titleHtml = warningIconHtml + titleHtml;
        }
      }

      titleElement.innerHTML = titleHtml;
      card.querySelector('.mod-card-summary').textContent = modData.summary || 'No summary available.';
      card.querySelector('.mod-card-author').innerHTML = `by <span class="author-name-highlight">${modData.author}</span>`;
      const currentLang = mapLangCode(languageSelector.value);
      const dateStr = formatNexusDate(modData.updated_timestamp, currentLang);
      card.querySelector('.mod-card-date').textContent = `Updated: ${dateStr}`;

      browseGridContainer.appendChild(card);
    }
  }

  async function openModDetailPanel(modData) {
    modDetailName.textContent = modData.name;
    modDetailName.dataset.modId = modData.mod_id;

    const imageUrl = modData.picture_url || '/src/assets/placeholder.png';

    if (imageUrl && imageUrl.startsWith('http')) {
      loadImageViaTauri(invoke, modDetailImage, imageUrl);
    } else {
      modDetailImage.src = imageUrl;
    }

    modDetailDescription.innerHTML = bbcodeToHtml(modData.description) || '<p>No description available.</p>';
    modDetailAuthor.textContent = modData.author || 'Unknown';
    modDetailVersion.textContent = modData.version || '?.?';

    const currentLang = mapLangCode(languageSelector.value);
    modDetailUpdated.textContent = formatNexusDate(modData.updated_timestamp, currentLang);
    modDetailCreated.textContent = formatNexusDate(modData.created_timestamp, currentLang);

    const modIdStr = String(modData.mod_id);
    const installedFiles = appState.installedModsMap.get(modIdStr);

    let versionToShow = 'N/A';

    if (installedFiles && installedFiles.size > 0) {
      let mainFileVersion = null;
      const allModFilesFromCurated = modData.files || [];

      for (const installedFileId of installedFiles.keys()) {
        const fileData = allModFilesFromCurated.find(f => String(f.file_id) === installedFileId);
        if (fileData && fileData.category_name === 'MAIN') {
          mainFileVersion = installedFiles.get(installedFileId);
          break;
        }
      }

      if (mainFileVersion) {
        versionToShow = mainFileVersion;
      } else {
        versionToShow = installedFiles.values().next().value || 'N/A';
      }
    }

    modDetailInstalled.textContent = versionToShow;

    modDetailInstallBtnContainer.innerHTML = '';
    const primaryBtn = document.createElement('button');
    primaryBtn.className = 'mod-card-install-btn';
    primaryBtn.textContent = (installedFiles && installedFiles.size > 0) ? 'MANAGE FILES' : 'DOWNLOAD';
    primaryBtn.onclick = () => showFileSelectionModal(modData.mod_id);
    modDetailInstallBtnContainer.appendChild(primaryBtn);

    modDetailSecondaryActions.innerHTML = '';
    const changelogBtn = document.createElement('button');
    changelogBtn.className = 'detail-action-btn';
    changelogBtn.textContent = 'Changelogs';
    changelogBtn.onclick = () => {
      const changelogs = modData.changelogs || {};
      displayChangelogs(modData.name, changelogs);
    };
    modDetailSecondaryActions.appendChild(changelogBtn);

    const nexusLinkBtn = document.createElement('a');
    nexusLinkBtn.className = 'detail-action-btn';
    nexusLinkBtn.textContent = 'Visit on Nexus';
    nexusLinkBtn.href = `https://www.nexusmods.com/nomanssky/mods/${modData.mod_id}`;
    nexusLinkBtn.target = '_blank';
    modDetailSecondaryActions.appendChild(nexusLinkBtn);

    if (!getIsPanelOpen()) {
      const screenWidth = window.screen.availWidth;

      if (screenWidth >= PANEL_OPEN_WIDTH) {
        setIsPanelOpen(true);
        const currentSize = await appWindow.innerSize();
        await appWindow.setSize(new LogicalSize(PANEL_OPEN_WIDTH, currentSize.height));
      } else {
        setIsPanelOpen(false);
      }
    }

    modDetailPanel.classList.add('open');
  }

  async function showFileSelectionModal(modId) {
    const curatedData = getCuratedData();
    const modData = curatedData.find(m => m.mod_id === modId);
    const filesData = { files: modData?.files || [] };

    if (!modData) {
      await window.customAlert('Could not find file information for this mod in the local data.', 'Error');
      return;
    }

    fileSelectionModalTitle.textContent = `Download: ${modData.name}`;
    fileSelectionListContainer.innerHTML = '';

    const modIdStr = String(modId);

    const createFileRow = (file) => {
      const item = document.createElement('div');
      item.className = 'update-item';

      let buttonHtml = '';
      const installedFilesForThisMod = appState.installedModsMap.get(modIdStr);
      const fileIdStr = String(file.file_id);
      const installedVersionForThisFile = installedFilesForThisMod ? installedFilesForThisMod.get(fileIdStr) : undefined;

      const rawFileName = file.file_name;
      const remoteBaseName = getBaseName(file.name || file.file_name);
      let replacingFileId = '';

      if (installedVersionForThisFile) {
        const isUpToDate = !isNewerVersionAvailable(installedVersionForThisFile, file.version);
        if (isUpToDate) {
          buttonHtml = '<button class="mod-card-install-btn" disabled>INSTALLED</button>';
        } else {
          replacingFileId = fileIdStr;
          buttonHtml = `<button class="mod-card-install-btn" data-file-id="${fileIdStr}" data-mod-id="${modId}" data-version="${file.version}" data-raw-filename="${rawFileName}" data-replacing-file-id="${replacingFileId}">UPDATE</button>`;
        }
      } else {
        let isUpdateForAnotherFile = false;

        if (installedFilesForThisMod) {
          for (const [installedFileId, installedVersion] of installedFilesForThisMod.entries()) {
            let installedBaseName = '';

            const installedNexusFile = filesData.files.find(f => String(f.file_id) === installedFileId);
            if (installedNexusFile) {
              installedBaseName = getBaseName(installedNexusFile.name || installedNexusFile.file_name);
            } else {
              for (const modEntry of appState.modDataCache.values()) {
                if (String(modEntry.local_info?.file_id) === installedFileId) {
                  if (modEntry.local_info.install_source) {
                    installedBaseName = getBaseName(modEntry.local_info.install_source);
                  }
                  break;
                }
              }
            }

            if (installedBaseName && installedBaseName === remoteBaseName) {
              if (isNewerVersionAvailable(installedVersion, file.version)) {
                isUpdateForAnotherFile = true;
                replacingFileId = installedFileId;
                break;
              }
            }
          }
        }

        const buttonText = isUpdateForAnotherFile ? 'UPDATE' : 'DOWNLOAD';
        buttonHtml = `<button class="mod-card-install-btn" data-file-id="${fileIdStr}" data-mod-id="${modId}" data-version="${file.version}" data-raw-filename="${rawFileName}" data-replacing-file-id="${replacingFileId}">${buttonText}</button>`;
      }

      const displayName = file.name || file.file_name;

      item.innerHTML = `
                <div class="update-item-info">
                    <div class="update-item-name">${displayName} (v${file.version})</div>
                    <div class="update-item-version">${file.description || 'No description.'}</div>
                </div>
                ${buttonHtml}`;
      return item;
    };

    const allowedCategories = ['MAIN', 'OPTIONAL', 'MISCELLANEOUS', 'OLD_VERSION'];
    const categorizedFiles = {};
    for (const file of filesData.files) {
      const category = file.category_name;
      if (allowedCategories.includes(category)) {
        if (!categorizedFiles[category]) categorizedFiles[category] = [];
        categorizedFiles[category].push(file);
      }
    }

    let primaryFileId = -1;
    if (categorizedFiles.MAIN && categorizedFiles.MAIN.length > 0) {
      categorizedFiles.MAIN.sort((a, b) => b.uploaded_timestamp - a.uploaded_timestamp);
      const primaryMainFile = categorizedFiles.MAIN[0];
      primaryFileId = primaryMainFile.file_id;
      const primaryContainer = document.createElement('div');
      primaryContainer.className = 'primary-file-container';
      primaryContainer.appendChild(createFileRow(primaryMainFile));
      fileSelectionListContainer.appendChild(primaryContainer);
    }

    const categoryOrder = ['OPTIONAL', 'MISCELLANEOUS', 'OLD_VERSION'];
    const collapsibleTemplate = document.getElementById('collapsibleSectionTemplate');
    const categoryDisplayNames = { OPTIONAL: 'Optional Files', MISCELLANEOUS: 'Miscellaneous', OLD_VERSION: 'Old Versions' };

    for (const category of categoryOrder) {
      if (categorizedFiles[category] && categorizedFiles[category].length > 0) {
        const section = collapsibleTemplate.content.cloneNode(true).firstElementChild;
        const header = section.querySelector('.collapsible-header');
        const content = section.querySelector('.collapsible-content');
        header.querySelector('.collapsible-title').textContent = categoryDisplayNames[category] || category;
        categorizedFiles[category].sort((a, b) => b.uploaded_timestamp - a.uploaded_timestamp);
        for (const file of categorizedFiles[category]) {
          if (file.file_id === primaryFileId) continue;
          content.appendChild(createFileRow(file));
        }
        if (content.hasChildNodes()) fileSelectionListContainer.appendChild(section);
      }
    }
    fileSelectionModalOverlay.classList.remove('hidden');
  }

  return {
    fetchAndRenderBrowseGrid,
    filterAndDisplayMods,
    openModDetailPanel,
    showFileSelectionModal,
  };
}
