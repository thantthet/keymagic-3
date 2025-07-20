const { invoke } = window.__TAURI__.core;
const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;

// Add error handling for Tauri API
if (!window.__TAURI__) {
  console.error('Tauri API not available!');
  alert('Error: Tauri API not available. Please restart the application.');
}

let bundledKeyboards = [];
let selectedKeyboards = new Set();

async function init() {
  try {
    console.log('Import wizard initializing...');
    
    // Get bundled keyboards comparison
    bundledKeyboards = await invoke('get_bundled_keyboards');
    console.log('Bundled keyboards:', bundledKeyboards);
    
    if (bundledKeyboards.length === 0) {
      // No bundled keyboards, just close
      console.log('No bundled keyboards found, closing wizard');
      await closeWizard();
      return;
    }
    
    // Setup UI
    const bundledList = document.getElementById('bundled-keyboards-list');
    const skipBtn = document.getElementById('import-wizard-skip');
    const importBtn = document.getElementById('import-wizard-import');
    
    // Clear previous selections
    selectedKeyboards.clear();
    
    // Render bundled keyboards
    bundledList.innerHTML = '';
    bundledKeyboards.forEach((comparison, index) => {
      const item = createComparisonItem(comparison, index);
      bundledList.appendChild(item);
    });
    
    // Update summary
    updateImportSummary();
    
    // Event listeners
    skipBtn.onclick = async (e) => {
      e.preventDefault();
      console.log('Skip button clicked');
      await closeWizard();
    };
    
    importBtn.onclick = async (e) => {
      e.preventDefault();
      console.log('Import button clicked');
      await importSelectedKeyboards();
    };
    
  } catch (error) {
    console.error('Failed to initialize import wizard:', error);
    await closeWizard();
  }
}

function createComparisonItem(comparison, index) {
  const item = document.createElement('div');
  item.className = 'keyboard-comparison-item';
  item.dataset.index = index;
  
  // Determine if this should be checked by default
  const shouldCheck = comparison.status === 'New' || comparison.status === 'Updated';
  if (shouldCheck) {
    selectedKeyboards.add(index);
    item.classList.add('selected');
  }
  
  // Status badge
  let statusBadge = '';
  let statusText = '';
  switch (comparison.status) {
    case 'New':
      statusBadge = '<span class="status-badge new">NEW</span>';
      statusText = 'Not installed';
      break;
    case 'Updated':
      statusBadge = '<span class="status-badge update">UPDATE</span>';
      statusText = 'Update available';
      break;
    case 'Unchanged':
      statusBadge = '<span class="status-badge current">CURRENT</span>';
      statusText = 'Already up to date';
      break;
    case 'Modified':
      statusBadge = '<span class="status-badge modified">MODIFIED</span>';
      statusText = 'Local file modified';
      break;
  }
  
  // Create icon
  let iconHtml = '';
  if (comparison.icon_data) {
    iconHtml = createIconElement(comparison.icon_data);
  } else {
    // Generate color based on name
    const color = generateColorFromString(comparison.name);
    iconHtml = createColoredIcon(color, comparison.name);
  }
  
  item.innerHTML = `
    <div class="comparison-checkbox">
      <input type="checkbox" id="kb-compare-${index}" ${shouldCheck ? 'checked' : ''} 
             ${comparison.status === 'Unchanged' ? 'disabled' : ''}>
    </div>
    <div class="comparison-icon">
      ${iconHtml}
    </div>
    <div class="comparison-info">
      <div class="comparison-name">${comparison.name}</div>
      <div class="comparison-status">
        ${statusBadge}
        <span>${statusText}</span>
      </div>
    </div>
  `;
  
  // Add click handler
  const checkbox = item.querySelector('input[type="checkbox"]');
  checkbox.addEventListener('change', (e) => {
    if (e.target.checked) {
      selectedKeyboards.add(index);
      item.classList.add('selected');
    } else {
      selectedKeyboards.delete(index);
      item.classList.remove('selected');
    }
    updateImportSummary();
  });
  
  // Click on item toggles checkbox
  item.addEventListener('click', (e) => {
    if (e.target.type !== 'checkbox' && !checkbox.disabled) {
      checkbox.checked = !checkbox.checked;
      checkbox.dispatchEvent(new Event('change'));
    }
  });
  
  return item;
}

function updateImportSummary() {
  const selectedCount = selectedKeyboards.size;
  const summaryDiv = document.getElementById('import-summary');
  const countSpan = document.getElementById('selected-count');
  const importBtn = document.getElementById('import-wizard-import');
  
  countSpan.textContent = selectedCount;
  summaryDiv.style.display = selectedCount > 0 ? 'block' : 'none';
  importBtn.disabled = selectedCount === 0;
  importBtn.textContent = selectedCount > 0 ? `Import ${selectedCount} Keyboard${selectedCount > 1 ? 's' : ''}` : 'Import Selected';
}

async function importSelectedKeyboards() {
  const importBtn = document.getElementById('import-wizard-import');
  importBtn.disabled = true;
  importBtn.textContent = 'Importing...';
  
  try {
    const results = [];
    
    for (const index of selectedKeyboards) {
      const keyboard = bundledKeyboards[index];
      try {
        await invoke('import_bundled_keyboard', { 
          bundledPath: keyboard.bundled_path,
          keyboardStatus: keyboard.status 
        });
        results.push({ name: keyboard.name, success: true });
      } catch (error) {
        console.error(`Failed to import ${keyboard.name}:`, error);
        results.push({ name: keyboard.name, success: false, error: error.toString() });
      }
    }
    
    // Clear the first run flag
    await invoke('clear_first_run_scan_keyboards');
    
    // Count successful imports
    const successCount = results.filter(r => r.success).length;
    
    // Emit event to main window before closing
    const { emit } = window.__TAURI__.event;
    await emit('keyboards-imported', { count: successCount });
    
    // Close wizard using same method as skip
    await closeWizard();
    
  } catch (error) {
    console.error('Failed to import keyboards:', error);
    alert('Failed to import keyboards: ' + error);
  } finally {
    importBtn.disabled = false;
    importBtn.textContent = 'Import Selected';
  }
}

async function closeWizard() {
  console.log('Closing wizard...');
  
  try {
    // Clear the first run flag
    await invoke('clear_first_run_scan_keyboards');
    console.log('Cleared first run flag');
  } catch (error) {
    console.error('Failed to clear first run flag:', error);
  }
  
  // Use getCurrentWebviewWindow() to get the current window and close it
  try {
    const currentWindow = getCurrentWebviewWindow();
    console.log('Closing window with getCurrentWebviewWindow().close()');
    await currentWindow.close();
  } catch (error) {
    console.error('Failed to close window:', error);
    // As a last resort, try window.close()
    try {
      window.close();
    } catch (e) {
      console.error('window.close() also failed:', e);
    }
  }
}

// Icon creation functions (same as main.js)
function createIconElement(iconData) {
  if (!iconData || (typeof iconData !== 'string' && iconData.length === 0)) {
    return createDefaultIcon();
  }
  
  // Handle both base64 string and raw bytes
  let base64;
  if (typeof iconData === 'string') {
    // Already base64 encoded
    base64 = iconData;
  } else {
    // Convert raw bytes to base64
    base64 = btoa(String.fromCharCode(...new Uint8Array(iconData)));
  }
  
  return `<img src="data:image/bmp;base64,${base64}" alt="Keyboard icon" style="width: 100%; height: 100%; object-fit: contain;">`;
}

function createColoredIcon(color, name) {
  const initial = name.charAt(0).toUpperCase();
  return `<div style="width: 100%; height: 100%; background-color: ${color}; display: flex; align-items: center; justify-content: center; border-radius: 8px; color: white; font-weight: bold; font-size: 20px;">${initial}</div>`;
}

function createDefaultIcon() {
  return `<svg width="32" height="32" viewBox="0 0 32 32" fill="currentColor">
    <rect x="4" y="10" width="24" height="16" rx="2" stroke="currentColor" stroke-width="1.5" fill="none"/>
    <rect x="8" y="14" width="3" height="3" fill="currentColor"/>
    <rect x="13" y="14" width="3" height="3" fill="currentColor"/>
    <rect x="18" y="14" width="3" height="3" fill="currentColor"/>
    <rect x="23" y="14" width="3" height="3" fill="currentColor"/>
    <rect x="8" y="19" width="3" height="3" fill="currentColor"/>
    <rect x="13" y="19" width="3" height="3" fill="currentColor"/>
    <rect x="18" y="19" width="3" height="3" fill="currentColor"/>
    <rect x="23" y="19" width="3" height="3" fill="currentColor"/>
  </svg>`;
}

function generateColorFromString(str) {
  const colors = [
    '#2196F3', '#4CAF50', '#FF9800', '#9C27B0', '#F44336',
    '#00BCD4', '#795548', '#607D8B', '#E91E63', '#009688',
    '#FFC107', '#3F51B5', '#8BC34A', '#FF5722', '#673AB7'
  ];
  
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  
  return colors[Math.abs(hash) % colors.length];
}

// Initialize when DOM is ready
window.addEventListener('DOMContentLoaded', init);