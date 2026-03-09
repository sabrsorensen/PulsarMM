/**
 * GamepadManager - Steam Deck / controller navigation for PulsarMM
 *
 * Manages gamepad input polling, focus ring navigation, section switching,
 * and on-screen button prompts. Emits custom DOM events for main.js to handle.
 */

import btnA from './assets/btn-a.png';
import btnB from './assets/btn-b.png';
import btnLB from './assets/btn-lb.png';
import btnRB from './assets/btn-rb.png';
import btnStart from './assets/btn-start.png';
import btnDpad from './assets/btn-dpad.png';

// Button indices (Xbox / Steam Deck standard mapping)
const BTN = {
  A: 0,
  B: 1,
  X: 2,
  Y: 3,
  LB: 4,
  RB: 5,
  BACK: 8,
  START: 9,
  DPAD_UP: 12,
  DPAD_DOWN: 13,
  DPAD_LEFT: 14,
  DPAD_RIGHT: 15,
};

// Axis indices
const AXIS = {
  LEFT_X: 0,
  LEFT_Y: 1,
};

const DEAD_ZONE = 0.3;
const REPEAT_INITIAL_MS = 250;
const REPEAT_INTERVAL_MS = 100;

export class GamepadManager {
  constructor() {
    this.connected = false;
    this.gamepadIndex = null;
    this.inputMode = 'mouse'; // 'mouse' | 'gamepad'
    this.polling = false;
    this.lastTimestamp = 0;

    // Focus state
    this.sections = [];
    this.currentSectionIndex = 0;
    this.currentItemIndex = 0;
    this.focusedElement = null;

    // Modal focus stack
    this.focusStack = [];

    // Button repeat tracking
    this.buttonState = {};
    this.axisState = { x: 0, y: 0 };

    // Prompt bar element references
    this.promptBar = null;
    this.promptConfigs = {};

    this._onGamepadConnected = this._onGamepadConnected.bind(this);
    this._onGamepadDisconnected = this._onGamepadDisconnected.bind(this);
    this._onMouseMove = this._onMouseMove.bind(this);
    this._poll = this._poll.bind(this);
  }

  init() {
    window.addEventListener('gamepadconnected', this._onGamepadConnected);
    window.addEventListener('gamepaddisconnected', this._onGamepadDisconnected);
    window.addEventListener('mousemove', this._onMouseMove);

    this.promptBar = document.getElementById('gamepadPromptBar');
    this._buildPromptBar();

    // Check if gamepad is already connected (e.g., page reload)
    const gamepads = navigator.getGamepads();
    for (let i = 0; i < gamepads.length; i++) {
      if (gamepads[i]) {
        this._onGamepadConnected({ gamepad: gamepads[i] });
        break;
      }
    }
  }

  destroy() {
    window.removeEventListener('gamepadconnected', this._onGamepadConnected);
    window.removeEventListener('gamepaddisconnected', this._onGamepadDisconnected);
    window.removeEventListener('mousemove', this._onMouseMove);
    this.polling = false;
    this._clearFocus();
  }

  // --- Section Registration ---

  /**
   * Register navigable sections. Call this after DOM renders.
   * @param {Array<{id: string, selector: string, container?: string, promptKey?: string}>} sectionDefs
   */
  setSections(sectionDefs) {
    this.sections = sectionDefs;
    // Clamp indices
    if (this.currentSectionIndex >= this.sections.length) {
      this.currentSectionIndex = 0;
    }
  }

  /**
   * Get focusable elements for a section
   */
  _getSectionElements(sectionIndex) {
    const section = this.sections[sectionIndex];
    if (!section) return [];
    const container = section.container
      ? document.querySelector(section.container)
      : document;
    if (!container) return [];
    const elements = Array.from(container.querySelectorAll(section.selector));
    // Filter out hidden elements
    return elements.filter(el => {
      if (el.offsetParent === null && el.style.position !== 'fixed') return false;
      if (el.closest('.hidden')) return false;
      return true;
    });
  }

  // --- Focus Ring ---

  _setFocus(element) {
    if (this.focusedElement) {
      this.focusedElement.classList.remove('gamepad-focus');
    }
    this.focusedElement = element;
    if (element) {
      element.classList.add('gamepad-focus');
      element.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      // Steam Deck: focusing text inputs helps SteamOS detect textbox context for OSK.
      if (this.inputMode === 'gamepad' && this._isTextEntryElement(element)) {
        this._focusTextEntry(element, false);
      }
    }
  }

  _clearFocus() {
    if (this.focusedElement) {
      this.focusedElement.classList.remove('gamepad-focus');
      this.focusedElement = null;
    }
  }

  _focusCurrentItem() {
    const elements = this._getSectionElements(this.currentSectionIndex);
    if (elements.length === 0) return;
    if (this.currentItemIndex >= elements.length) {
      this.currentItemIndex = elements.length - 1;
    }
    if (this.currentItemIndex < 0) {
      this.currentItemIndex = 0;
    }
    this._setFocus(elements[this.currentItemIndex]);
  }

  // --- Modal Focus Trapping ---

  pushModalFocus(modalSelector, itemSelector) {
    this.focusStack.push({
      sectionIndex: this.currentSectionIndex,
      itemIndex: this.currentItemIndex,
      sections: this.sections,
    });
    this.sections = [{ id: 'modal', selector: itemSelector, container: modalSelector, promptKey: 'modal' }];
    this.currentSectionIndex = 0;
    this.currentItemIndex = 0;
    this._focusCurrentItem();
    this._updatePrompts();
  }

  popModalFocus() {
    if (this.focusStack.length === 0) return;
    const saved = this.focusStack.pop();
    this.sections = saved.sections;
    this.currentSectionIndex = saved.sectionIndex;
    this.currentItemIndex = saved.itemIndex;
    this._focusCurrentItem();
    this._updatePrompts();
  }

  // --- Prompt Bar ---

  _buildPromptBar() {
    if (!this.promptBar) return;
    // Define prompt sets
    this.promptConfigs = {
      mods: [
        { icon: btnDpad, label: 'Navigate' },
        { icon: btnA, label: 'Select' },
        { icon: btnB, label: 'Back' },
        { icon: btnLB, label: 'Prev Section' },
        { icon: btnRB, label: 'Next Section' },
        { icon: btnStart, label: 'Launch' },
      ],
      browse: [
        { icon: btnDpad, label: 'Navigate' },
        { icon: btnA, label: 'Details' },
        { icon: btnB, label: 'Back' },
        { icon: btnLB, label: 'Prev Page' },
        { icon: btnRB, label: 'Next Page' },
      ],
      modal: [
        { icon: btnDpad, label: 'Navigate' },
        { icon: btnA, label: 'Confirm' },
        { icon: btnB, label: 'Cancel' },
      ],
    };
    this._updatePrompts();
  }

  _updatePrompts() {
    if (!this.promptBar) return;
    const section = this.sections[this.currentSectionIndex];
    const key = section?.promptKey || 'mods';
    const config = this.promptConfigs[key] || this.promptConfigs.mods;

    this.promptBar.innerHTML = config.map(item =>
      `<div class="gamepad-prompt-item">
        <img src="${item.icon}" class="gamepad-prompt-icon" alt="${item.label}">
        <span>${item.label}</span>
      </div>`
    ).join('');
  }

  // --- Input Mode ---

  _switchToGamepad() {
    if (this.inputMode === 'gamepad') return;
    this.inputMode = 'gamepad';
    document.body.classList.remove('gamepad-inactive');
    document.body.classList.add('gamepad-active');
    // Restore focus if we have none
    if (!this.focusedElement) {
      this._focusCurrentItem();
    }
    this._updatePrompts();
  }

  _switchToMouse() {
    if (this.inputMode === 'mouse') return;
    this.inputMode = 'mouse';
    document.body.classList.remove('gamepad-active');
    document.body.classList.add('gamepad-inactive');
    this._clearFocus();
  }

  _onMouseMove() {
    this._switchToMouse();
  }

  // --- Gamepad Connection ---

  _onGamepadConnected(e) {
    this.connected = true;
    this.gamepadIndex = e.gamepad.index;
    console.log(`Gamepad connected: ${e.gamepad.id}`);
    if (!this.polling) {
      this.polling = true;
      requestAnimationFrame(this._poll);
    }
  }

  _onGamepadDisconnected(e) {
    if (e.gamepad.index === this.gamepadIndex) {
      this.connected = false;
      this.gamepadIndex = null;
      this.polling = false;
      this._switchToMouse();
      console.log('Gamepad disconnected');
    }
  }

  // --- Polling Loop ---

  _poll(timestamp) {
    if (!this.polling || !this.connected) return;
    requestAnimationFrame(this._poll);

    const gp = navigator.getGamepads()[this.gamepadIndex];
    if (!gp) return;

    // Process buttons
    this._processButton(gp, BTN.A, 'select', timestamp);
    this._processButton(gp, BTN.B, 'back', timestamp);
    this._processButton(gp, BTN.LB, 'section-prev', timestamp);
    this._processButton(gp, BTN.RB, 'section-next', timestamp);
    this._processButton(gp, BTN.START, 'launch', timestamp);
    this._processButton(gp, BTN.DPAD_UP, 'nav-up', timestamp);
    this._processButton(gp, BTN.DPAD_DOWN, 'nav-down', timestamp);
    this._processButton(gp, BTN.DPAD_LEFT, 'nav-left', timestamp);
    this._processButton(gp, BTN.DPAD_RIGHT, 'nav-right', timestamp);

    // Process left stick as D-pad equivalent
    const lx = gp.axes[AXIS.LEFT_X] || 0;
    const ly = gp.axes[AXIS.LEFT_Y] || 0;
    this._processAxis(ly, 'stick-y', 'nav-up', 'nav-down', timestamp);
    this._processAxis(lx, 'stick-x', 'nav-left', 'nav-right', timestamp);
  }

  _processButton(gp, btnIndex, action, timestamp) {
    const pressed = gp.buttons[btnIndex]?.pressed || false;
    const state = this.buttonState[action] || { held: false, nextRepeat: 0 };

    if (pressed && !state.held) {
      // Initial press
      state.held = true;
      state.nextRepeat = timestamp + REPEAT_INITIAL_MS;
      this._handleAction(action);
    } else if (pressed && state.held && timestamp >= state.nextRepeat) {
      // Repeat
      const isRepeatable = action.startsWith('nav-');
      if (isRepeatable) {
        state.nextRepeat = timestamp + REPEAT_INTERVAL_MS;
        this._handleAction(action);
      }
    } else if (!pressed) {
      state.held = false;
    }

    this.buttonState[action] = state;
  }

  _processAxis(value, axisKey, negAction, posAction, timestamp) {
    const state = this.buttonState[axisKey] || { held: false, nextRepeat: 0, direction: 0 };
    const direction = Math.abs(value) > DEAD_ZONE ? Math.sign(value) : 0;

    if (direction !== 0 && (state.direction !== direction || !state.held)) {
      state.held = true;
      state.direction = direction;
      state.nextRepeat = timestamp + REPEAT_INITIAL_MS;
      this._handleAction(direction < 0 ? negAction : posAction);
    } else if (direction !== 0 && state.held && timestamp >= state.nextRepeat) {
      state.nextRepeat = timestamp + REPEAT_INTERVAL_MS;
      this._handleAction(direction < 0 ? negAction : posAction);
    } else if (direction === 0) {
      state.held = false;
      state.direction = 0;
    }

    this.buttonState[axisKey] = state;
  }

  // --- Action Handling ---

  _handleAction(action) {
    this._switchToGamepad();

    switch (action) {
      case 'nav-up':
        this._navigate(-1, 'vertical');
        break;
      case 'nav-down':
        this._navigate(1, 'vertical');
        break;
      case 'nav-left':
        this._navigate(-1, 'horizontal');
        break;
      case 'nav-right':
        this._navigate(1, 'horizontal');
        break;
      case 'select':
        this._activateFocused();
        break;
      case 'back':
        this._dispatchEvent('gamepad-back');
        break;
      case 'section-prev':
        this._changeSection(-1);
        break;
      case 'section-next':
        this._changeSection(1);
        break;
      case 'launch':
        this._dispatchEvent('gamepad-launch');
        break;
    }
  }

  _navigate(delta, axis) {
    const elements = this._getSectionElements(this.currentSectionIndex);
    if (elements.length === 0) return;

    if (axis === 'horizontal') {
      // Keep mod-row horizontal behavior (toggle) but allow left/right focus navigation
      // for regular controls such as tabs/buttons/selects.
      const focused = this.focusedElement;
      if (focused && focused.classList.contains('mod-row')) {
        this._dispatchEvent('gamepad-navigate-horizontal', { delta });
        return;
      }
    }

    this.currentItemIndex += delta;
    if (this.currentItemIndex < 0) this.currentItemIndex = 0;
    if (this.currentItemIndex >= elements.length) this.currentItemIndex = elements.length - 1;
    this._setFocus(elements[this.currentItemIndex]);
  }

  _changeSection(delta) {
    if (this.sections.length === 0) return;
    const prevIndex = this.currentSectionIndex;
    this.currentSectionIndex += delta;

    // Wrap around
    if (this.currentSectionIndex < 0) this.currentSectionIndex = this.sections.length - 1;
    if (this.currentSectionIndex >= this.sections.length) this.currentSectionIndex = 0;

    // Skip sections with no visible elements
    let attempts = this.sections.length;
    while (attempts > 0) {
      const elements = this._getSectionElements(this.currentSectionIndex);
      if (elements.length > 0) break;
      this.currentSectionIndex += delta > 0 ? 1 : -1;
      if (this.currentSectionIndex < 0) this.currentSectionIndex = this.sections.length - 1;
      if (this.currentSectionIndex >= this.sections.length) this.currentSectionIndex = 0;
      attempts--;
    }

    if (this.currentSectionIndex !== prevIndex) {
      this.currentItemIndex = 0;
    }
    this._focusCurrentItem();
    this._updatePrompts();
  }

  _activateFocused() {
    if (this.focusedElement) {
      // Text entry gets explicit focus so SteamOS can open on-screen keyboard.
      if (this._isTextEntryElement(this.focusedElement)) {
        this._focusTextEntry(this.focusedElement, true);
        this._dispatchEvent('gamepad-text-input-focus', { element: this.focusedElement });
        return;
      }

      this.focusedElement.click();
      this._dispatchEvent('gamepad-select', { element: this.focusedElement });
    }
  }

  _isTextEntryElement(element) {
    if (!element || element.disabled || element.readOnly) return false;
    if (element.tagName === 'TEXTAREA') return true;
    if (element.tagName === 'INPUT') {
      const type = (element.type || 'text').toLowerCase();
      return ['text', 'search', 'email', 'url', 'tel', 'number', 'password'].includes(type);
    }
    return element.isContentEditable || element.getAttribute('role') === 'textbox';
  }

  _focusTextEntry(element, placeCaretAtEnd) {
    element.focus({ preventScroll: true });
    if (placeCaretAtEnd && typeof element.value === 'string' && typeof element.setSelectionRange === 'function') {
      const end = element.value.length;
      element.setSelectionRange(end, end);
    }
    element.click();
  }

  _dispatchEvent(name, detail = {}) {
    window.dispatchEvent(new CustomEvent(name, { detail }));
  }
}
