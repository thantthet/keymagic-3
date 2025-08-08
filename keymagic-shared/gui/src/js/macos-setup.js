// macOS IMK setup handler
const { invoke } = window.__TAURI__.core;

let setupDialog = null;

// Check if we're on macOS
export async function checkMacOSSetup(platformInfo) {
    if (!platformInfo || platformInfo.os !== 'macos') {
        return;
    }

    try {
        const imkInfo = await invoke('check_imk_status');
        
        // Show setup if not installed or needs update
        if (!imkInfo.installed || imkInfo.needs_update) {
            showSetupDialog(imkInfo);
        }
    } catch (error) {
        console.error('Failed to check IMK status:', error);
    }
}

function showSetupDialog(imkInfo) {
    // Remove existing dialog if any
    if (setupDialog) {
        setupDialog.remove();
    }

    // Create dialog HTML
    const dialogHTML = `
        <div id="macos-setup-dialog" class="modal-overlay">
            <div class="modal-content">
                <div class="modal-header">
                    <h2>${!imkInfo.installed ? 'Input Method Setup Required' : 'Input Method Update Available'}</h2>
                </div>
                <div class="modal-body">
                    <div class="alert alert-info">
                        <svg class="alert-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10"></circle>
                            <line x1="12" y1="16" x2="12" y2="12"></line>
                            <line x1="12" y1="8" x2="12.01" y2="8"></line>
                        </svg>
                        <div>
                            ${!imkInfo.installed ? 
                                'KeyMagic needs to install its input method component to work properly. This is a one-time setup that allows you to type using KeyMagic keyboards.' :
                                'A newer version of the KeyMagic input method is available. Update to get the latest features and improvements.'
                            }
                        </div>
                    </div>
                    <div id="setup-result" style="display: none;"></div>
                </div>
                <div class="modal-footer">
                    <button id="setup-install-btn" class="btn btn-primary">
                        ${!imkInfo.installed ? 'Install Input Method' : 'Update Input Method'}
                    </button>
                </div>
            </div>
        </div>
    `;

    // Add dialog to document
    setupDialog = document.createElement('div');
    setupDialog.innerHTML = dialogHTML;
    document.body.appendChild(setupDialog);

    // Add event listeners
    document.getElementById('setup-install-btn').addEventListener('click', () => {
        installIMK();
    });

    // Add styles if not already present
    if (!document.getElementById('macos-setup-styles')) {
        const styles = document.createElement('style');
        styles.id = 'macos-setup-styles';
        styles.textContent = `
            .modal-overlay {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background-color: rgba(0, 0, 0, 0.5);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 9999;
            }

            .modal-content {
                background: var(--bg-primary, #ffffff);
                border-radius: 8px;
                width: 90%;
                max-width: 500px;
                box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
                border: 1px solid var(--border-color, #e5e7eb);
            }

            .modal-header {
                padding: 20px;
                border-bottom: 1px solid var(--border-color, #e5e7eb);
                background: var(--bg-secondary, #f9fafb);
                border-radius: 8px 8px 0 0;
            }

            .modal-header h2 {
                margin: 0;
                font-size: 20px;
                font-weight: 600;
            }

            .modal-body {
                padding: 20px;
            }

            .modal-footer {
                padding: 20px;
                border-top: 1px solid var(--border-color, #e5e7eb);
                display: flex;
                justify-content: center;
                gap: 10px;
            }

            .alert {
                display: flex;
                gap: 12px;
                padding: 16px;
                border-radius: 6px;
                margin-bottom: 16px;
            }

            .alert-info {
                background-color: rgba(59, 130, 246, 0.1);
                color: var(--text-primary, #1f2937);
                border: 1px solid rgba(59, 130, 246, 0.2);
            }

            .alert-success {
                background-color: rgba(34, 197, 94, 0.1);
                color: var(--text-primary, #1f2937);
                border: 1px solid rgba(34, 197, 94, 0.2);
            }

            .alert-error {
                background-color: rgba(239, 68, 68, 0.1);
                color: var(--text-primary, #1f2937);
                border: 1px solid rgba(239, 68, 68, 0.2);
            }

            .alert-icon {
                width: 20px;
                height: 20px;
                flex-shrink: 0;
            }

            .alert-info .alert-icon {
                color: #3b82f6;
            }

            .alert-success .alert-icon {
                color: #22c55e;
            }

            .alert-error .alert-icon {
                color: #ef4444;
            }
        `;
        document.head.appendChild(styles);
    }
}

async function installIMK() {
    const installBtn = document.getElementById('setup-install-btn');
    const resultDiv = document.getElementById('setup-result');
    
    // Disable button and show loading
    installBtn.disabled = true;
    installBtn.textContent = 'Installing...';
    
    try {
        const result = await invoke('install_imk_bundle');
        
        // Show result
        resultDiv.style.display = 'block';
        resultDiv.className = result.success ? 'alert alert-success' : 'alert alert-error';
        
        // Convert newlines to <br> for proper formatting
        const formattedMessage = result.message.replace(/\n/g, '<br>');
        
        resultDiv.innerHTML = `
            <svg class="alert-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                ${result.success ? 
                    '<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline>' :
                    '<circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line>'
                }
            </svg>
            <div>
                <strong>${result.success ? 'Success!' : 'Error'}</strong><br>
                ${formattedMessage}
                ${result.requires_logout ? '<br><br><strong>Please log out and log back in for changes to take full effect.</strong>' : ''}
                ${result.success && !result.already_enabled ? '<br><br><button id="open-settings-btn" class="btn btn-primary">Open System Settings</button>' : ''}
            </div>
        `;
        
        // Update buttons
        if (result.success) {
            installBtn.textContent = 'Close';
            installBtn.disabled = false;
            installBtn.onclick = () => {
                setupDialog.remove();
                setupDialog = null;
            };
            
            // Add event listener for Open Settings button
            const openSettingsBtn = document.getElementById('open-settings-btn');
            if (openSettingsBtn) {
                openSettingsBtn.addEventListener('click', async () => {
                    try {
                        await invoke('open_input_sources_settings');
                    } catch (error) {
                        console.error('Failed to open settings:', error);
                    }
                });
            }
        } else {
            installBtn.textContent = 'Retry';
            installBtn.disabled = false;
        }
    } catch (error) {
        // Show error
        resultDiv.style.display = 'block';
        resultDiv.className = 'alert alert-error';
        resultDiv.innerHTML = `
            <svg class="alert-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="15" y1="9" x2="9" y2="15"></line>
                <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
            <div>
                <strong>Installation Failed</strong><br>
                ${error}
            </div>
        `;
        
        installBtn.textContent = 'Retry';
        installBtn.disabled = false;
    }
}