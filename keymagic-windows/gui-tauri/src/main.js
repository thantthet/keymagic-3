const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// State management
let keyboards = [];
let activeKeyboardId = null;
let selectedKeyboardId = null;
let keyProcessingEnabled = true;

// DOM Elements
let toggleButton;
let statusDot;
let keyboardList;
let addKeyboardBtn;
let modal;

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
  
  // Load version for about page
  if (pageName === 'about') {
    loadAboutVersion();
  }
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
        ${keyboard.icon_data ? createIconElement(keyboard.icon_data) : 
          keyboard.color ? createColoredIcon(keyboard.color, keyboard.name) : 
          createDefaultIcon()}
      </div>
      <div class="keyboard-info">
        <div class="keyboard-name">${keyboard.name}</div>
        <div class="keyboard-description">${keyboard.description || 'No description'}</div>
      </div>
    </div>
    <div class="keyboard-meta">
      <span class="keyboard-status ${isActive ? 'active' : ''}">
        ${isActive ? 'Active' : 'Inactive'}
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
        `<button class="btn btn-disabled" disabled>Active</button>`
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

function createColoredIcon(color, name) {
  // Get the first letter of the name (or first character)
  const firstChar = name.charAt(0).toUpperCase();
  
  return `
    <div style="
      width: 100%;
      height: 100%;
      background-color: ${color};
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
      font-size: 20px;
      font-weight: bold;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    ">
      ${firstChar}
    </div>
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

// Event listener setup function
function setupEventListeners() {
  // Add keyboard
  addKeyboardBtn.addEventListener('click', async () => {
    try {
      // Use the invoke command to call the dialog through Tauri
      const selected = await invoke('plugin:dialog|open', {
        options: {
          multiple: false,
          filters: [{
            name: 'KeyMagic Keyboard',
            extensions: ['km2']
          }]
        }
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

  
  // Modal event listeners
  modal.querySelector('.modal-close').addEventListener('click', hideModal);
  modal.addEventListener('click', (e) => {
    if (e.target === modal) {
      hideModal();
    }
  });
}

function updateStatusIndicator() {
  if (!statusDot || !toggleButton) return;
  
  if (keyProcessingEnabled) {
    statusDot.classList.remove('disabled');
    toggleButton.querySelector('span:last-child').textContent = 'Disable KeyMagic';
  } else {
    statusDot.classList.add('disabled');
    toggleButton.querySelector('span:last-child').textContent = 'Enable KeyMagic';
  }
}

// Settings
async function loadSettings() {
  try {
    const startWithWindows = await invoke('get_setting', { key: 'StartWithWindows' });
    
    document.getElementById('start-with-windows').checked = startWithWindows === '1';
    
    // Load on/off hotkey
    const onOffHotkey = await invoke('get_on_off_hotkey');
    if (onOffHotkey) {
      document.getElementById('on-off-hotkey').value = onOffHotkey;
    }
    
    // Load current version
    await loadCurrentVersion();
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}


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

// Toast notification functions
let toastContainer = null;

function ensureToastContainer() {
  if (!toastContainer) {
    toastContainer = document.createElement('div');
    toastContainer.className = 'toast-container';
    document.body.appendChild(toastContainer);
  }
  return toastContainer;
}

function showToast(message, type = 'info', duration = 3000) {
  const container = ensureToastContainer();
  
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  
  // Add icon based on type
  const icon = type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ';
  
  toast.innerHTML = `
    <span class="toast-icon">${icon}</span>
    <span class="toast-message">${message}</span>
    <button class="toast-close" onclick="this.parentElement.remove()">×</button>
  `;
  
  container.appendChild(toast);
  
  // Trigger reflow to enable transition
  toast.offsetHeight;
  
  // Add show class for animation
  toast.classList.add('show');
  
  // Auto-remove after duration
  const timeoutId = setTimeout(() => {
    toast.classList.remove('show');
    setTimeout(() => toast.remove(), 300);
  }, duration);
  
  // Clear timeout if manually closed
  toast.querySelector('.toast-close').addEventListener('click', () => {
    clearTimeout(timeoutId);
  });
}

function showSuccess(message) {
  showToast(message, 'success');
}

function showError(message) {
  showToast(message, 'error');
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

// On/Off hotkey configuration
let currentOnOffHotkey = null;

window.configureOnOffHotkey = function() {
  recordedKeys = [];
  
  // Get current hotkey
  const currentValue = document.getElementById('on-off-hotkey').value;
  if (currentValue) {
    recordedKeys = currentValue.split('+');
  }
  
  showModal(
    'Configure On/Off Hotkey',
    `
      <p>Configure global hotkey to enable/disable KeyMagic</p>
      <div class="hotkey-input-container">
        <input type="text" id="hotkey-input" class="hotkey-input" 
               placeholder="Press key combination or leave empty..." 
               value="${currentValue}"
               readonly>
        <button class="btn btn-secondary" onclick="clearHotkey()">Clear</button>
      </div>
      <p class="hotkey-hint">Press the desired key combination (e.g., Ctrl+Shift+Space) or click Clear to remove hotkey</p>
    `,
    `
      <button class="btn btn-secondary" onclick="cancelOnOffHotkeyConfig()">Cancel</button>
      <button class="btn btn-primary" onclick="saveOnOffHotkey()">Save</button>
    `
  );
  
  // Focus on the input and set up key listeners
  setTimeout(() => {
    const input = document.getElementById('hotkey-input');
    input.focus();
    input.addEventListener('keydown', recordHotkey);
  }, 100);
}

window.cancelOnOffHotkeyConfig = function() {
  const input = document.getElementById('hotkey-input');
  if (input) {
    input.removeEventListener('keydown', recordHotkey);
  }
  recordedKeys = [];
  hideModal();
}

window.saveOnOffHotkey = async function() {
  const input = document.getElementById('hotkey-input');
  if (input) {
    input.removeEventListener('keydown', recordHotkey);
  }
  
  try {
    let hotkeyValue;
    let successMessage;
    
    if (recordedKeys.length === 0) {
      // User cleared the hotkey - send null to remove it
      hotkeyValue = null;
      successMessage = 'On/Off hotkey removed';
    } else {
      // User set a hotkey
      hotkeyValue = recordedKeys.join('+');
      successMessage = 'On/Off hotkey configured';
    }
    
    await invoke('set_on_off_hotkey', {
      hotkey: hotkeyValue
    });
    
    // Update the display
    document.getElementById('on-off-hotkey').value = hotkeyValue || '';
    
    showSuccess(successMessage);
  } catch (error) {
    console.error('Failed to save on/off hotkey:', error);
    showError('Failed to save on/off hotkey');
  }
  
  recordedKeys = [];
  hideModal();
}

// Update checking functionality
let updateInfo = null;

window.checkForUpdates = async function() {
  const button = document.querySelector('button[onclick="checkForUpdates()"]');
  const statusElement = document.getElementById('update-status');
  const downloadBtn = document.getElementById('download-update-btn');
  const updateNotes = document.getElementById('update-notes');
  const releaseNotesContent = document.getElementById('release-notes-content');
  
  // Show loading state
  button.disabled = true;
  button.textContent = 'Checking...';
  statusElement.textContent = 'Checking for updates...';
  statusElement.className = 'update-status checking';
  
  try {
    updateInfo = await invoke('check_for_update');
    
    // Update current version display
    document.getElementById('current-version').textContent = updateInfo.current_version;
    
    if (updateInfo.update_available) {
      statusElement.textContent = `New version ${updateInfo.latest_version} is available!`;
      statusElement.className = 'update-status available';
      
      // Show download button if download URL is available
      if (updateInfo.download_url) {
        downloadBtn.style.display = 'inline-block';
      }
      
      // Show release notes if available
      if (updateInfo.release_notes) {
        updateNotes.style.display = 'block';
        // Simple text rendering without markdown parsing
        releaseNotesContent.textContent = updateInfo.release_notes;
      }
    } else {
      statusElement.textContent = 'You are using the latest version.';
      statusElement.className = 'update-status up-to-date';
      downloadBtn.style.display = 'none';
      updateNotes.style.display = 'none';
    }
  } catch (error) {
    console.error('Failed to check for updates:', error);
    statusElement.textContent = 'Failed to check for updates. Please try again later.';
    statusElement.className = 'update-status error';
    downloadBtn.style.display = 'none';
    updateNotes.style.display = 'none';
  } finally {
    button.disabled = false;
    button.textContent = 'Check for Updates';
  }
}

window.downloadUpdate = async function() {
  if (updateInfo && updateInfo.download_url) {
    try {
      const { open } = window.__TAURI__.shell;
      await open(updateInfo.download_url);
    } catch (error) {
      console.error('Failed to open download URL:', error);
      showError('Failed to open download page');
    }
  }
}

// Load current version on settings page
async function loadCurrentVersion() {
  // Don't load version here - it will be loaded when checking for updates
  // Just ensure the element shows "-" initially
  const currentVersionElement = document.getElementById('current-version');
  if (currentVersionElement) {
    currentVersionElement.textContent = '-';
  }
}

// Load version on about page
async function loadAboutVersion() {
  try {
    const aboutVersionElement = document.getElementById('about-version');
    if (aboutVersionElement) {
      const version = await invoke('get_app_version');
      aboutVersionElement.textContent = version;
    }
  } catch (error) {
    console.error('Failed to load about version:', error);
    const aboutVersionElement = document.getElementById('about-version');
    if (aboutVersionElement) {
      aboutVersionElement.textContent = '-';
    }
  }
}

// Show update notification
function showUpdateNotification(updateInfo) {
  // Create a subtle notification banner
  const banner = document.createElement('div');
  banner.className = 'update-notification-banner';
  banner.innerHTML = `
    <div class="update-notification-content">
      <span>New version ${updateInfo.latest_version} is available!</span>
      <button class="btn-link" onclick="switchPage('settings'); checkForUpdates(); this.parentElement.parentElement.remove();">
        View Details
      </button>
      <button class="btn-link" onclick="this.parentElement.parentElement.remove();">
        Dismiss
      </button>
    </div>
  `;
  
  // Add to the body
  document.body.appendChild(banner);
  
  // Auto-dismiss after 10 seconds
  setTimeout(() => {
    if (banner.parentElement) {
      banner.remove();
    }
  }, 10000);
}

// Initialize
async function init() {
  // Initialize DOM elements
  toggleButton = document.getElementById('toggle-keymagic');
  statusDot = document.querySelector('.toggle-button .status-dot');
  keyboardList = document.getElementById('keyboard-list');
  addKeyboardBtn = document.getElementById('add-keyboard-btn');
  modal = document.getElementById('modal');
  
  // Set up event listeners
  setupEventListeners();
  
  try {
    keyProcessingEnabled = await invoke('is_key_processing_enabled');
    updateStatusIndicator();
    await loadKeyboards();
    await loadSettings();
    
    // Load version for about page if it's the active page
    const activeNavItem = document.querySelector('.nav-item.active');
    if (activeNavItem && activeNavItem.dataset.page === 'about') {
      await loadAboutVersion();
    }
    
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
  
  // Listen for keyboard switched events from hotkeys
  await listen('keyboard-switched', async (event) => {
    activeKeyboardId = event.payload;
    await loadKeyboards(); // Reload to get latest state
    // TODO: Show HUD notification when implemented
  });
  
  // Listen for check for updates event from tray
  await listen('check_for_updates', async () => {
    // Delay slightly to ensure settings page is loaded
    setTimeout(() => {
      checkForUpdates();
    }, 100);
  });
  
  // Listen for update available notification
  await listen('update_available', async (event) => {
    const updateInfo = event.payload;
    // Show a subtle notification in the UI
    showUpdateNotification(updateInfo);
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