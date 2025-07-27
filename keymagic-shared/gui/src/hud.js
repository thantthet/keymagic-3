const { listen } = window.__TAURI__.event;
const { getCurrentWebviewWindow } = window.__TAURI__.webviewWindow;

let hideTimeout;

async function showNotification(payload) {
    const hudElement = document.getElementById('hud');
    const iconContainer = document.getElementById('iconContainer');
    const defaultIcon = document.getElementById('defaultIcon');
    const customIcon = document.getElementById('customIcon');
    const textElement = document.getElementById('text');
    
    // Clear any existing timeout
    if (hideTimeout) {
        clearTimeout(hideTimeout);
    }
    
    // Remove hiding class if present
    hudElement.classList.remove('hiding');
    
    // Set content
    textElement.textContent = payload.text || '';
    
    // Set icon
    if (payload.icon) {
        // Show custom icon
        customIcon.src = `data:image/png;base64,${payload.icon}`;
        customIcon.style.display = 'block';
        defaultIcon.style.display = 'none';
        iconContainer.classList.remove('default-icon');
    } else {
        // Show default keyboard icon
        customIcon.style.display = 'none';
        defaultIcon.style.display = 'block';
        iconContainer.classList.add('default-icon');
    }
    
    // Show the window
    const window = getCurrentWebviewWindow();
    await window.show();
    await window.setFocus();
    
    // Auto-hide after delay
    hideTimeout = setTimeout(async () => {
        hudElement.classList.add('hiding');
        setTimeout(async () => {
            await window.hide();
            hudElement.classList.remove('hiding');
        }, 200);
    }, payload.duration || 1500);
}

// Listen for show-notification events
listen('show-notification', (event) => {
    showNotification(event.payload);
});

// Hide on click (optional)
document.addEventListener('click', async () => {
    if (hideTimeout) {
        clearTimeout(hideTimeout);
    }
    const hudElement = document.getElementById('hud');
    const window = getCurrentWebviewWindow();
    hudElement.classList.add('hiding');
    setTimeout(async () => {
        await window.hide();
        hudElement.classList.remove('hiding');
    }, 200);
});