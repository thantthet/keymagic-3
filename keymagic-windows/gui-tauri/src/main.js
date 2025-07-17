const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// State management
let keyboards = [];
let activeKeyboardId = null;
let selectedKeyboardId = null;
// DOM Elements
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
  
  // Load composition mode settings for settings page
  if (pageName === 'settings') {
    loadCompositionModeProcesses();
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


// Settings
async function loadSettings() {
  try {
    const startWithWindows = await invoke('get_setting', { key: 'StartWithWindows' });
    
    document.getElementById('start-with-windows').checked = startWithWindows === '1';
    
    
    // Load current version
    await loadCurrentVersion();
    
    // Load composition mode processes
    await loadCompositionModeProcesses();
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

// Composition Mode Process Management
async function loadCompositionModeProcesses() {
  try {
    const processes = await invoke('get_composition_mode_processes');
    renderProcessList(processes);
  } catch (error) {
    console.error('Failed to load composition mode processes:', error);
    showError('Failed to load composition mode processes');
  }
}

function renderProcessList(processes) {
  const processList = document.getElementById('composition-mode-process-list');
  if (!processList) return;
  
  processList.innerHTML = '';
  
  if (processes.length === 0) {
    processList.innerHTML = `
      <div class="process-list-empty">
        <p>No applications configured for composition mode.<br>
        All applications will use direct input mode.</p>
      </div>
    `;
    return;
  }
  
  processes.forEach(processName => {
    const item = document.createElement('div');
    item.className = 'process-item';
    item.innerHTML = `
      <span class="process-name">${processName}</span>
      <button class="btn-remove" onclick="removeProcessFromCompositionMode('${processName.replace(/'/g, "\\'")}')">Remove</button>
    `;
    processList.appendChild(item);
  });
}

async function addProcessToCompositionMode() {
  const processName = await showProcessInputDialog();
  if (!processName) return;
  
  try {
    await invoke('add_composition_mode_process', { processName });
    await loadCompositionModeProcesses();
  } catch (error) {
    console.error('Failed to add process:', error);
    showError('Failed to add process to composition mode');
  }
}

async function removeProcessFromCompositionMode(processName) {
  try {
    await invoke('remove_composition_mode_process', { processName });
    await loadCompositionModeProcesses();
  } catch (error) {
    console.error('Failed to remove process:', error);
    showError('Failed to remove process from composition mode');
  }
}

function showProcessInputDialog() {
  return new Promise((resolve) => {
    // Store resolve function for later use
    window._processInputResolve = resolve;
    
    showModal(
      'Add Application',
      `
        <p>Enter the executable name of the application (e.g., "ms-teams.exe"):</p>
        <input type="text" id="process-name-input" class="modal-input" placeholder="application.exe" 
               autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" />
        <p class="modal-hint">The application will use composition mode (underlined text while typing).</p>
      `,
      `
        <button class="btn btn-secondary" onclick="hideModal(); window.cancelAddProcess();">Cancel</button>
        <button class="btn btn-primary" onclick="window.confirmAddProcess();">Add</button>
      `
    );
    
    // Focus the input field
    setTimeout(() => {
      const input = document.getElementById('process-name-input');
      if (input) {
        input.focus();
        input.addEventListener('keydown', (e) => {
          if (e.key === 'Enter') {
            window.confirmAddProcess();
          } else if (e.key === 'Escape') {
            hideModal();
            window.cancelAddProcess();
          }
        });
      }
    }, 100);
  });
}

window.confirmAddProcess = function() {
  const input = document.getElementById('process-name-input');
  const processName = input ? input.value.trim() : '';
  
  if (processName) {
    hideModal();
    if (window._processInputResolve) {
      window._processInputResolve(processName);
      delete window._processInputResolve;
    }
  }
};

window.cancelAddProcess = function() {
  if (window._processInputResolve) {
    window._processInputResolve(null);
    delete window._processInputResolve;
  }
};

// Make these functions available globally
window.addProcessToCompositionMode = addProcessToCompositionMode;
window.removeProcessFromCompositionMode = removeProcessFromCompositionMode;


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

// Function to open Windows input settings
window.openWindowsInputSettings = async function() {
  try {
    // Use rundll32 to open the language settings
    await invoke('run_command', {
      command: 'rundll32.exe',
      args: ['Shell32.dll,Control_RunDLL', 'input.dll,,{C07337D3-DB2C-4D0B-9A93-B722A6C106E2}']
    });
  } catch (error) {
    console.error('Failed to open Windows input settings:', error);
    showError('Failed to open Windows input settings');
  }
}

// Update checking functionality
let updateInfo = null;

window.checkForUpdates = async function() {
  const button = document.querySelector('button[onclick="checkForUpdates()"]');
  const statusElement = document.getElementById('update-status');
  const statusContainer = document.getElementById('update-status-container');
  
  // Show loading state
  button.disabled = true;
  button.textContent = 'Checking...';
  statusContainer.style.display = 'block';
  statusElement.textContent = 'Checking for updates...';
  statusElement.className = 'update-status checking';
  
  try {
    updateInfo = await invoke('check_for_update');
    
    // Update current version display
    document.getElementById('current-version').textContent = updateInfo.current_version;
    
    if (updateInfo.update_available) {
      statusElement.textContent = `New version ${updateInfo.latest_version} is available!`;
      statusElement.className = 'update-status available';
      
      // Open update window
      await showUpdateWindow(updateInfo);
    } else {
      statusElement.textContent = 'You are using the latest version.';
      statusElement.className = 'update-status up-to-date';
    }
  } catch (error) {
    console.error('Failed to check for updates:', error);
    statusElement.textContent = 'Failed to check for updates. Please try again later.';
    statusElement.className = 'update-status error';
  } finally {
    button.disabled = false;
    button.textContent = 'Check for Updates';
  }
}

// Show update window
window.showUpdateWindow = async function(updateInfo) {
  try {
    // Check if we should show the update (remind me later logic)
    const remindAfterStr = await invoke('get_setting', { key: 'UpdateRemindAfter' });
    if (remindAfterStr) {
      const remindAfter = parseInt(remindAfterStr);
      if (!isNaN(remindAfter) && Date.now() < remindAfter) {
        // Still in "remind later" period, don't show window
        console.log('Update window skipped due to remind later setting');
        return;
      }
    }
    
    // Create update window
    const { WebviewWindow } = window.__TAURI__.webviewWindow;
    
    const updateWindow = new WebviewWindow('update-window', {
      url: `update-window.html?updateInfo=${encodeURIComponent(JSON.stringify(updateInfo))}`,
      title: 'KeyMagic Update Available',
      width: 600,
      height: 450,
      center: true,
      resizable: true,
      alwaysOnTop: false,
      decorations: true,
      transparent: false,
      maximizable: false,
      minimizable: true,
    });
    
  } catch (error) {
    console.error('Failed to show update window:', error);
  }
}

// Load current version on settings page
async function loadCurrentVersion() {
  try {
    const currentVersionElement = document.getElementById('current-version');
    if (currentVersionElement) {
      const version = await invoke('get_app_version');
      currentVersionElement.textContent = version;
    }
  } catch (error) {
    console.error('Failed to load current version:', error);
    const currentVersionElement = document.getElementById('current-version');
    if (currentVersionElement) {
      currentVersionElement.textContent = '-';
    }
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
      <button class="btn-link" onclick="window.showUpdateWindow(${JSON.stringify(updateInfo).replace(/"/g, '&quot;')}); this.parentElement.parentElement.remove();">
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

// Language profile management
async function loadLanguageProfiles() {
  try {
    // Get supported languages
    const supportedLanguages = await invoke('get_supported_languages');
    
    // Get currently enabled languages
    const enabledLanguages = await invoke('get_enabled_languages');
    
    // Populate language list
    const languageList = document.getElementById('language-list');
    languageList.innerHTML = '';
    
    supportedLanguages.forEach(([code, name]) => {
      const item = document.createElement('div');
      item.className = 'language-item';
      
      const label = document.createElement('label');
      
      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.value = code;
      checkbox.checked = enabledLanguages.includes(code);
      checkbox.addEventListener('change', onLanguageToggle);
      
      const text = document.createElement('span');
      text.textContent = name;
      
      label.appendChild(checkbox);
      label.appendChild(text);
      item.appendChild(label);
      languageList.appendChild(item);
    });
  } catch (error) {
    console.error('Failed to load language profiles:', error);
  }
}

async function onLanguageToggle(event) {
  try {
    // Get all checked languages
    const checkboxes = document.querySelectorAll('#language-list input[type="checkbox"]:checked');
    const enabledLanguages = Array.from(checkboxes).map(cb => cb.value);
    
    // Ensure at least one language is selected
    if (enabledLanguages.length === 0) {
      event.target.checked = true;
      showError('At least one language must be selected');
      return;
    }
    
    // Update the enabled languages
    await invoke('set_enabled_languages', { languages: enabledLanguages });
    
    // Show success message
    showSuccess('Language profiles updated successfully');
  } catch (error) {
    console.error('Failed to update language profiles:', error);
    showError('Failed to update language profiles');
    // Restore checkbox state on error
    await loadLanguageProfiles();
  }
}

// Initialize
async function init() {
  // Initialize DOM elements
  keyboardList = document.getElementById('keyboard-list');
  addKeyboardBtn = document.getElementById('add-keyboard-btn');
  modal = document.getElementById('modal');
  
  // Set up event listeners
  setupEventListeners();
  
  // Listen for keyboard import events
  const { listen } = window.__TAURI__.event;
  listen('keyboards-imported', async (event) => {
    console.log('Keyboards imported event received:', event);
    await loadKeyboards();
  });
  
  try {
    await loadKeyboards();
    await loadSettings();
    await loadLanguageProfiles();
    
    // Check for first run keyboard scan
    await checkFirstRunImport();
    
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
    // Show update window directly on automatic check
    await showUpdateWindow(updateInfo);
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
// Import Wizard Functions
async function checkFirstRunImport() {
  try {
    const shouldShowWizard = await invoke('check_first_run_scan_keyboards');
    if (shouldShowWizard) {
      // Check if there are actually keyboards to import
      const bundledKeyboards = await invoke('get_bundled_keyboards');
      
      // Filter for keyboards that need importing (new or updated)
      const keyboardsToImport = bundledKeyboards.filter(kb => 
        kb.status === 'New' || kb.status === 'Updated'
      );
      
      if (keyboardsToImport.length > 0) {
        // Show wizard only if there are keyboards to import
        await showImportWizard();
      } else {
        // No keyboards to import, clear the flag without showing wizard
        await invoke('clear_first_run_scan_keyboards');
        console.log('No keyboards to import, skipping wizard');
      }
    }
  } catch (error) {
    console.error('Failed to check first run:', error);
  }
}

async function showImportWizard() {
  try {
    // Create a new window for the import wizard
    const { WebviewWindow } = window.__TAURI__.webviewWindow;
    
    const wizardWindow = new WebviewWindow('import-wizard', {
      url: 'import-wizard.html',
      title: 'Keyboard Import Wizard',
      width: 600,
      height: 500,
      center: true,
      resizable: true,
      skipTaskbar: false,
      alwaysOnTop: false,
      decorations: true,
      transparent: false,
      maximizable: false,
      minimizable: false,
    });
    
    // Listen for window close to reload keyboards
    wizardWindow.once('tauri://closed', async () => {
      console.log('Import wizard closed, reloading keyboards...');
      // Reload keyboards after wizard closes
      await loadKeyboards();
      console.log('Keyboards reloaded');
    });
    
  } catch (error) {
    console.error('Failed to show import wizard:', error);
    showError('Failed to open import wizard: ' + error.message);
  }
}

window.addEventListener('DOMContentLoaded', init);