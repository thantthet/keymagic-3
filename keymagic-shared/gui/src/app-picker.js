const { invoke } = window.__TAURI__.core;
const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;
const { emit } = window.__TAURI__.event;

// Add error handling for Tauri API
if (!window.__TAURI__) {
  console.error('Tauri API not available!');
  alert('Error: Tauri API not available. Please restart the application.');
}

let selectedAppId = null;
let currentMode = null;
let platformInfo = null;
let allApps = [];

async function init() {
  console.log('App picker initializing...');
  
  // Get mode from URL params
  const params = new URLSearchParams(window.location.search);
  currentMode = params.get('mode') || 'composition';
  console.log('Mode:', currentMode);
  
  // Update description based on mode
  const description = document.getElementById('mode-description');
  if (currentMode === 'composition') {
    description.textContent = 'Select applications that will use composition mode (underlined text while typing)';
  } else {
    description.textContent = 'Select applications that will use direct mode (immediate text input)';
  }
  
  // Get platform info
  try {
    platformInfo = await invoke('get_platform_info');
    console.log('Platform info:', platformInfo);
    updatePlaceholder();
  } catch (error) {
    console.error('Failed to get platform info:', error);
  }
  
  // Load running apps
  await loadApps();
  
  // Set up event listeners
  document.getElementById('app-search').addEventListener('input', (e) => {
    filterApps(e.target.value);
  });
  
  document.getElementById('manual-app-id').addEventListener('input', (e) => {
    if (e.target.value.trim()) {
      // Clear selection when typing manually
      clearSelection();
    }
  });
  
  document.getElementById('cancel-btn').addEventListener('click', () => {
    getCurrentWebviewWindow().close();
  });
  
  document.getElementById('add-btn').addEventListener('click', addApplication);
  
  // Handle Enter key
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      addApplication();
    } else if (e.key === 'Escape') {
      getCurrentWebviewWindow().close();
    }
  });
  
  // Focus search input
  document.getElementById('app-search').focus();
}

function updatePlaceholder() {
  const manualInput = document.getElementById('manual-app-id');
  if (platformInfo) {
    if (platformInfo.os === 'windows') {
      manualInput.placeholder = 'Enter exe name (e.g., notepad.exe)';
    } else if (platformInfo.os === 'macos') {
      manualInput.placeholder = 'Enter bundle ID (e.g., com.apple.Safari)';
    } else {
      manualInput.placeholder = 'Enter application identifier';
    }
  }
}

async function loadApps() {
  try {
    console.log('Loading running apps...');
    allApps = await invoke('get_running_apps');
    console.log('Loaded apps:', allApps);
    displayApps(allApps);
  } catch (error) {
    console.error('Failed to load running apps:', error);
    displayError();
  }
}

function displayApps(apps) {
  const appList = document.getElementById('app-list');
  const emptyMessage = document.getElementById('app-list-empty');
  
  if (apps.length === 0) {
    appList.style.display = 'none';
    emptyMessage.style.display = 'block';
    return;
  }
  
  appList.style.display = 'block';
  emptyMessage.style.display = 'none';
  
  appList.innerHTML = '';
  
  apps.forEach(app => {
    const item = document.createElement('div');
    item.className = 'app-item';
    item.dataset.appId = app.identifier;
    item.dataset.appName = app.display_name.toLowerCase();
    
    // Create icon element
    let iconElement;
    if (app.icon_base64) {
      iconElement = `<img src="data:image/png;base64,${app.icon_base64}" class="app-icon" alt="${app.display_name}">`;
    } else {
      // Create placeholder icon with first letter
      const firstLetter = app.display_name.charAt(0).toUpperCase();
      iconElement = `<div class="app-icon placeholder">${firstLetter}</div>`;
    }
    
    item.innerHTML = `
      ${iconElement}
      <div class="app-info">
        <div class="app-name">${app.display_name}</div>
        <div class="app-id">${app.identifier}</div>
      </div>
    `;
    
    item.addEventListener('click', () => selectApp(app.identifier, item));
    
    appList.appendChild(item);
  });
}

function displayError() {
  const appList = document.getElementById('app-list');
  appList.innerHTML = `
    <div class="app-list-empty">
      <p>Failed to load applications.</p>
      <p>You can still enter an identifier manually below.</p>
    </div>
  `;
}

function selectApp(appId, element) {
  clearSelection();
  
  // Select new item
  element.classList.add('selected');
  selectedAppId = appId;
  
  // Clear manual input when selecting from list
  document.getElementById('manual-app-id').value = '';
}

function clearSelection() {
  document.querySelectorAll('.app-item').forEach(item => {
    item.classList.remove('selected');
  });
  selectedAppId = null;
}

function filterApps(searchTerm) {
  const term = searchTerm.toLowerCase();
  const filteredApps = allApps.filter(app => {
    const name = app.display_name.toLowerCase();
    const id = app.identifier.toLowerCase();
    return name.includes(term) || id.includes(term);
  });
  
  displayApps(filteredApps);
}

async function addApplication() {
  // Get selected app ID (from list or manual input)
  const manualId = document.getElementById('manual-app-id').value.trim();
  const appId = manualId || selectedAppId;
  
  if (!appId) {
    alert('Please select an application or enter an identifier');
    return;
  }
  
  try {
    if (currentMode === 'composition') {
      await invoke('add_composition_mode_host', { hostName: appId });
    } else if (currentMode === 'direct') {
      await invoke('add_direct_mode_host', { hostName: appId });
    }
    
    // Emit event to notify main window
    await emit('app-added', { mode: currentMode, appId });
    
    // Close window
    getCurrentWebviewWindow().close();
  } catch (error) {
    console.error('Failed to add application:', error);
    alert('Failed to add application: ' + error);
  }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}