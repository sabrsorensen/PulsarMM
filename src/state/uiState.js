export const DEFAULT_WIDTH = 950;
export const PANEL_OPEN_WIDTH = 1300;
export const SCROLL_SPEED = 5;

export function createAppState() {
  return {
    gamePath: null,
    settingsPath: null,
    versionType: null,
    currentFilePath: null,
    activeProfile: 'Default',
    selectedProfileView: 'Default',
    currentPage: 1,
    modsPerPage: 20,
    nexusUsername: null,
    isProfileSwitching: false,
    xmlDoc: null,
    isPopulating: false,
    currentTranslations: {},
    selectedModRow: null,
    installedModsMap: new Map(),
    modDataCache: new Map(),
    selectedModNames: new Set(),
    selectedDownloadIds: new Set(),
  };
}

export function createDragState() {
  return {
    draggedElement: null,
    ghostElement: null,
    placeholder: null,
    offsetX: 0,
    offsetY: 0,
    originalNextSibling: null,
    dragTimer: null,
    selectedModNameBeforeDrag: null,
  };
}

export function createScrollState() {
  return {
    isScrollingUp: false,
    isScrollingDown: false,
    animationFrameId: null,
  };
}

export function createDownloadSortState() {
  return {
    key: 'date',
    direction: 'desc',
  };
}
