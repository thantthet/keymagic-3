const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { listen } = window.__TAURI__.event;

// State management
let keyboards = [];
let activeKeyboardId = null;
let selectedKeyboardId = null;
let keyProcessingEnabled = true;

// DOM Elements
const statusIndicator = document.getElementById('status-indicator');
const statusDot = statusIndicator.querySelector('.status-dot');
const statusText = statusIndicator.querySelector('.status-text');
const toggleButton = document.getElementById('toggle-keymagic');
const keyboardList = document.getElementById('keyboard-list');
const addKeyboardBtn = document.getElementById('add-keyboard-btn');
const modal = document.getElementById('modal');

// Navigation
document.querySelectorAll('.nav-item').forEach(item => {
  item.addEventListener('click', () => {
    const page = item.dataset.page;
    switchPage(page);
  });
});

function switchPage(pageName) {
  // Update navigation
  document.querySelectorAll('.nav-item').forEach(item => {
    item.classList.toggle('active', item.dataset.page === pageName);
  });
  
  // Update pages
  document.querySelectorAll('.page').forEach(page => {
    page.classList.toggle('active', page.id === `${pageName}-page`);
  });
}

// Keyboard Management
async function loadKeyboards() {
  try {
    keyboards = await invoke('get_keyboards');
    activeKeyboardId = await invoke('get_active_keyboard');
    renderKeyboardList();
  } catch (error) {
    console.error('Failed to load keyboards:', error);
    showError('Failed to load keyboards');
  }
}

function renderKeyboardList() {
  keyboardList.innerHTML = '';
  
  keyboards.forEach(keyboard => {
    const card = createKeyboardCard(keyboard);
    keyboardList.appendChild(card);
  });
  
  if (keyboards.length === 0) {
    keyboardList.innerHTML = `
      <div style="text-align: center; padding: 40px; color: var(--text-secondary);">
        <p>No keyboards installed</p>
        <p style="margin-top: 10px;">Click "Add Keyboard" to install a keyboard layout</p>
      </div>
    `;
  }
}

function createKeyboardCard(keyboard) {
  const isActive = keyboard.id === activeKeyboardId;
  const isSelected = keyboard.id === selectedKeyboardId;
  
  const card = document.createElement('div');
  card.className = `keyboard-card ${isActive ? 'active' : ''} ${isSelected ? 'selected' : ''}`;
  card.dataset.keyboardId = keyboard.id;
  
  card.innerHTML = `
    <div class="keyboard-header">
      <div class="keyboard-icon">
        ${keyboard.icon_data ? createIconElement(keyboard.icon_data) : createDefaultIcon()}
      </div>
      <div class="keyboard-info">
        <div class="keyboard-name">${keyboard.name}</div>
        <div class="keyboard-description">${keyboard.description || 'No description'}</div>
      </div>
    </div>
    <div class="keyboard-meta">
      <span class="keyboard-status ${keyboard.enabled ? 'active' : 'disabled'}">
        ${keyboard.enabled ? 'Active' : 'Disabled'}
      </span>
      ${(() => {
        // Determine what hotkey to display
        let displayHotkey = '';
        let displayClass = 'keyboard-hotkey clickable';
        let displayTitle = 'Click to configure hotkey';
        
        if (keyboard.hotkey !== null && keyboard.hotkey !== undefined) {
          // Custom hotkey is explicitly set (could be empty string)
          if (keyboard.hotkey === '') {
            displayHotkey = 'No hotkey';
            displayClass += ' add-hotkey';
          } else {
            displayHotkey = keyboard.hotkey;
          }
        } else if (keyboard.default_hotkey) {
          // No custom hotkey, use default
          displayHotkey = keyboard.default_hotkey;
          displayTitle = 'Default hotkey - Click to configure';
        } else {
          // No custom hotkey and no default
          displayHotkey = 'No hotkey';
          displayClass += ' add-hotkey';
        }
        
        return `<span class="${displayClass}" onclick="configureHotkey('${keyboard.id}')" title="${displayTitle}">${displayHotkey}</span>`;
      })()}
    </div>
    <div class="keyboard-actions">
      ${!isActive ? 
        `<button class="btn btn-primary" onclick="activateKeyboard('${keyboard.id}')">Activate</button>` :
        `<button class="btn btn-secondary" disabled>Active</button>`
      }
      <button class="btn btn-secondary" onclick="removeKeyboard('${keyboard.id}')">Remove</button>
    </div>
  `;
  
  card.addEventListener('click', (e) => {
    if (!e.target.closest('button')) {
      selectKeyboard(keyboard.id);
    }
  });
  
  return card;
}

function createIconElement(iconData) {
  // Convert icon data to base64 and create img element
  const base64 = btoa(String.fromCharCode(...new Uint8Array(iconData)));
  return `<img src="data:image/png;base64,${base64}" alt="Keyboard icon" style="width: 100%; height: 100%; object-fit: contain;">`;
}

function createDefaultIcon() {
  return `
    <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
      <path d="M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm9 7H8v-2h8v2zm0-4h-2v-2h2v2zm0-3h-2V8h2v2zm3 3h-2v-2h2v2zm0-3h-2V8h2v2z"/>
    </svg>
  `;
}

function selectKeyboard(keyboardId) {
  selectedKeyboardId = keyboardId;
  renderKeyboardList();
}

window.activateKeyboard = async function(keyboardId) {
  try {
    await invoke('set_active_keyboard', { keyboardId });
    activeKeyboardId = keyboardId;
    renderKeyboardList();
    await updateTrayMenu();
    showSuccess('Keyboard activated');
  } catch (error) {
    console.error('Failed to activate keyboard:', error);
    showError('Failed to activate keyboard');
  }
}

window.removeKeyboard = async function(keyboardId) {
  const keyboard = keyboards.find(k => k.id === keyboardId);
  if (!keyboard) return;
  
  const confirmed = await showConfirmDialog(
    'Remove Keyboard',
    `Are you sure you want to remove "${keyboard.name}"?`
  );
  
  if (confirmed) {
    try {
      await invoke('remove_keyboard', { keyboardId });
      await loadKeyboards();
      await updateTrayMenu();
      showSuccess('Keyboard removed');
    } catch (error) {
      console.error('Failed to remove keyboard:', error);
      showError('Failed to remove keyboard');
    }
  }
}

// Add keyboard
addKeyboardBtn.addEventListener('click', async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'KeyMagic Keyboard',
        extensions: ['km2']
      }]
    });
    
    if (selected) {
      try {
        const keyboardId = await invoke('add_keyboard', { path: selected });
        await loadKeyboards();
        await updateTrayMenu();
        showSuccess('Keyboard added successfully');
      } catch (error) {
        console.error('Failed to add keyboard:', error);
        showError('Failed to add keyboard: ' + error);
      }
    }
  } catch (error) {
    console.error('Failed to open file dialog:', error);
    showError('Failed to open file dialog');
  }
});

// Toggle KeyMagic
toggleButton.addEventListener('click', async () => {
  try {
    const newState = !keyProcessingEnabled;
    await invoke('set_key_processing_enabled', { enabled: newState });
    keyProcessingEnabled = newState;
    updateStatusIndicator();
    await updateTrayMenu();
    showSuccess(newState ? 'KeyMagic enabled' : 'KeyMagic disabled');
  } catch (error) {
    console.error('Failed to toggle KeyMagic:', error);
    showError('Failed to toggle KeyMagic');
  }
});

function updateStatusIndicator() {
  if (keyProcessingEnabled) {
    statusDot.classList.remove('disabled');
    statusText.textContent = 'Enabled';
    toggleButton.querySelector('span').textContent = 'Disable KeyMagic';
  } else {
    statusDot.classList.add('disabled');
    statusText.textContent = 'Disabled';
    toggleButton.querySelector('span').textContent = 'Enable KeyMagic';
  }
}

// Settings
async function loadSettings() {
  try {
    const startWithWindows = await invoke('get_setting', { key: 'StartWithWindows' });
    const showInTray = await invoke('get_setting', { key: 'ShowInSystemTray' });
    const minimizeToTray = await invoke('get_setting', { key: 'MinimizeToTray' });
    
    document.getElementById('start-with-windows').checked = startWithWindows === '1';
    document.getElementById('show-in-tray').checked = showInTray !== '0';
    document.getElementById('minimize-to-tray').checked = minimizeToTray === '1';
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

// Setting change handlers
document.getElementById('start-with-windows').addEventListener('change', async (e) => {
  try {
    await invoke('set_setting', { key: 'StartWithWindows', value: e.target.checked ? '1' : '0' });
    showSuccess('Setting saved');
  } catch (error) {
    console.error('Failed to save setting:', error);
    showError('Failed to save setting');
    e.target.checked = !e.target.checked;
  }
});

document.getElementById('show-in-tray').addEventListener('change', async (e) => {
  try {
    await invoke('set_setting', { key: 'ShowInSystemTray', value: e.target.checked ? '1' : '0' });
    showSuccess('Setting saved');
  } catch (error) {
    console.error('Failed to save setting:', error);
    showError('Failed to save setting');
    e.target.checked = !e.target.checked;
  }
});

document.getElementById('minimize-to-tray').addEventListener('change', async (e) => {
  try {
    await invoke('set_setting', { key: 'MinimizeToTray', value: e.target.checked ? '1' : '0' });
    showSuccess('Setting saved');
  } catch (error) {
    console.error('Failed to save setting:', error);
    showError('Failed to save setting');
    e.target.checked = !e.target.checked;
  }
});

// Modal functions
function showModal(title, content, footer) {
  const modalTitle = modal.querySelector('.modal-title');
  const modalBody = modal.querySelector('.modal-body');
  const modalFooter = modal.querySelector('.modal-footer');
  
  modalTitle.textContent = title;
  modalBody.innerHTML = content;
  modalFooter.innerHTML = footer || '';
  
  modal.classList.add('show');
}

window.hideModal = function() {
  modal.classList.remove('show');
}

modal.querySelector('.modal-close').addEventListener('click', hideModal);
modal.addEventListener('click', (e) => {
  if (e.target === modal) {
    hideModal();
  }
});

async function showConfirmDialog(title, message) {
  return new Promise(resolve => {
    showModal(
      title,
      `<p>${message}</p>`,
      `
        <button class="btn btn-secondary" onclick="hideModal(); window.confirmResolve(false);">Cancel</button>
        <button class="btn btn-primary" onclick="hideModal(); window.confirmResolve(true);">Confirm</button>
      `
    );
    window.confirmResolve = resolve;
  });
}

function showSuccess(message) {
  // TODO: Implement toast notifications
  console.log('Success:', message);
}

function showError(message) {
  // TODO: Implement toast notifications
  console.error('Error:', message);
}

// Hotkey configuration
let currentHotkeyKeyboard = null;
let recordedKeys = [];

window.configureHotkey = function(keyboardId) {
  const keyboard = keyboards.find(k => k.id === keyboardId);
  if (!keyboard) return;
  
  currentHotkeyKeyboard = keyboard;
  recordedKeys = [];
  
  // Determine initial display value and state
  let initialValue = '';
  let statusText = '';
  
  if (keyboard.hotkey !== null && keyboard.hotkey !== undefined) {
    // Custom hotkey is set
    if (keyboard.hotkey === '') {
      // Explicitly no hotkey
      initialValue = '';
      statusText = '<span style="color: var(--warning-color)">Currently: No hotkey (explicitly disabled)</span>';
    } else {
      // Custom hotkey
      initialValue = keyboard.hotkey;
      recordedKeys = keyboard.hotkey.split('+');
      statusText = '<span style="color: var(--success-color)">Currently: Custom hotkey</span>';
    }
  } else if (keyboard.default_hotkey) {
    // Using default hotkey
    initialValue = keyboard.default_hotkey;
    recordedKeys = keyboard.default_hotkey.split('+');
    statusText = '<span style="color: var(--primary-color)">Currently: Using default hotkey</span>';
  } else {
    // No hotkey at all
    initialValue = '';
    statusText = '<span style="color: var(--text-secondary)">Currently: No hotkey available</span>';
  }
  
  showModal(
    'Configure Hotkey',
    `
      <p>Configuring hotkey for: <strong>${keyboard.name}</strong></p>
      ${statusText}
      <div class="hotkey-input-container">
        <input type="text" id="hotkey-input" class="hotkey-input" 
               placeholder="Press key combination or leave empty..." 
               value="${initialValue}"
               readonly>
        <button class="btn btn-secondary" onclick="clearHotkey()">Clear</button>
        ${keyboard.default_hotkey ? 
          `<button class="btn btn-secondary" onclick="useDefaultHotkey()">Use Default</button>` : 
          ''
        }
      </div>
      <p class="hotkey-hint">Press the desired key combination (e.g., Ctrl+Shift+M) or click Clear to remove hotkey</p>
      ${keyboard.default_hotkey ? 
        `<p class="hotkey-default">Default hotkey: ${keyboard.default_hotkey}</p>` : 
        '<p class="hotkey-default">No default hotkey available</p>'}
    `,
    `
      <button class="btn btn-secondary" onclick="cancelHotkeyConfig()">Cancel</button>
      <button class="btn btn-primary" onclick="saveHotkey()">Save</button>
    `
  );
  
  // Focus on the input and set up key listeners
  setTimeout(() => {
    const input = document.getElementById('hotkey-input');
    input.focus();
    input.addEventListener('keydown', recordHotkey);
  }, 100);
}

function recordHotkey(e) {
  e.preventDefault();
  e.stopPropagation();
  
  recordedKeys = [];
  
  // Record modifiers
  if (e.ctrlKey) recordedKeys.push('Ctrl');
  if (e.shiftKey) recordedKeys.push('Shift');
  if (e.altKey) recordedKeys.push('Alt');
  
  // Record the main key
  if (e.key && e.key.length === 1) {
    // Single character key
    recordedKeys.push(e.key.toUpperCase());
  } else if (e.code) {
    // Special keys
    const keyMap = {
      'F1': 'F1', 'F2': 'F2', 'F3': 'F3', 'F4': 'F4',
      'F5': 'F5', 'F6': 'F6', 'F7': 'F7', 'F8': 'F8',
      'F9': 'F9', 'F10': 'F10', 'F11': 'F11', 'F12': 'F12',
      'Space': 'Space', 'Enter': 'Enter', 'Tab': 'Tab',
      'Escape': 'Esc', 'Backspace': 'Backspace', 'Delete': 'Delete',
      'Home': 'Home', 'End': 'End', 'PageUp': 'PageUp', 'PageDown': 'PageDown',
      'ArrowUp': 'Up', 'ArrowDown': 'Down', 'ArrowLeft': 'Left', 'ArrowRight': 'Right'
    };
    
    for (const [code, name] of Object.entries(keyMap)) {
      if (e.code === code || e.key === code) {
        recordedKeys.push(name);
        break;
      }
    }
    
    // Handle digit keys
    if (e.code.startsWith('Digit')) {
      recordedKeys.push(e.code.replace('Digit', ''));
    } else if (e.code.startsWith('Key')) {
      recordedKeys.push(e.code.replace('Key', ''));
    }
  }
  
  // Update the input display
  const hotkeyString = recordedKeys.join('+');
  document.getElementById('hotkey-input').value = hotkeyString;
}

window.clearHotkey = function() {
  recordedKeys = [];
  document.getElementById('hotkey-input').value = '';
}

window.useDefaultHotkey = function() {
  if (currentHotkeyKeyboard && currentHotkeyKeyboard.default_hotkey) {
    // Set special marker to indicate we want to use default
    recordedKeys = ['USE_DEFAULT'];
    document.getElementById('hotkey-input').value = currentHotkeyKeyboard.default_hotkey;
  }
}

window.cancelHotkeyConfig = function() {
  const input = document.getElementById('hotkey-input');
  if (input) {
    input.removeEventListener('keydown', recordHotkey);
  }
  currentHotkeyKeyboard = null;
  recordedKeys = [];
  hideModal();
}

window.saveHotkey = async function() {
  const input = document.getElementById('hotkey-input');
  if (input) {
    input.removeEventListener('keydown', recordHotkey);
  }
  
  if (currentHotkeyKeyboard) {
    try {
      let hotkeyValue;
      let successMessage;
      
      if (recordedKeys.length === 1 && recordedKeys[0] === 'USE_DEFAULT') {
        // User wants to use default hotkey - send null
        hotkeyValue = null;
        successMessage = 'Restored default hotkey';
      } else if (recordedKeys.length === 0) {
        // User cleared the hotkey - send empty string
        hotkeyValue = "";
        successMessage = 'Hotkey removed';
      } else {
        // User set a custom hotkey
        hotkeyValue = recordedKeys.join('+');
        successMessage = 'Hotkey configured';
      }
      
      await invoke('set_keyboard_hotkey', {
        keyboardId: currentHotkeyKeyboard.id,
        hotkey: hotkeyValue
      });
      
      // Update local data
      currentHotkeyKeyboard.hotkey = hotkeyValue;
      renderKeyboardList();
      await updateTrayMenu();
      
      showSuccess(successMessage);
    } catch (error) {
      console.error('Failed to save hotkey:', error);
      showError('Failed to save hotkey');
    }
  }
  
  currentHotkeyKeyboard = null;
  recordedKeys = [];
  hideModal();
}

// Initialize
async function init() {
  try {
    keyProcessingEnabled = await invoke('is_key_processing_enabled');
    updateStatusIndicator();
    await loadKeyboards();
    await loadSettings();
    
    // Set up event listeners for system tray interactions
    await setupTrayEventListeners();
  } catch (error) {
    console.error('Failed to initialize:', error);
    showError('Failed to initialize application');
  }
}

// System tray event listeners
async function setupTrayEventListeners() {
  // Listen for navigation events from system tray
  await listen('navigate', (event) => {
    const page = event.payload;
    switchPage(page);
  });
  
  // Listen for key processing state changes from tray
  await listen('key_processing_changed', async (event) => {
    keyProcessingEnabled = event.payload;
    updateStatusIndicator();
  });
  
  // Listen for active keyboard changes from tray
  await listen('active_keyboard_changed', async (event) => {
    activeKeyboardId = event.payload;
    renderKeyboardList();
  });
}

// Update tray menu when keyboards change
async function updateTrayMenu() {
  try {
    await invoke('update_tray_menu');
  } catch (error) {
    console.error('Failed to update tray menu:', error);
  }
}

// Start the app when DOM is loaded
window.addEventListener('DOMContentLoaded', init);