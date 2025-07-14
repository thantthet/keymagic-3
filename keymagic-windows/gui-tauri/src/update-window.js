const { invoke } = window.__TAURI__.core;
const { WebviewWindow } = window.__TAURI__.webviewWindow;
const { open } = window.__TAURI__.opener;

let updateInfo = null;

// Simple markdown parser
function parseMarkdown(markdown) {
  if (!markdown) return '';
  
  let html = markdown;
  
  // Headers
  html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
  html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
  html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');
  
  // Bold
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/__(.+?)__/g, '<strong>$1</strong>');
  
  // Italic
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');
  html = html.replace(/_(.+?)_/g, '<em>$1</em>');
  
  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank">$1</a>');
  
  // Lists
  html = html.replace(/^\* (.+)$/gim, '<li>$1</li>');
  html = html.replace(/^- (.+)$/gim, '<li>$1</li>');
  html = html.replace(/^\d+\. (.+)$/gim, '<li>$1</li>');
  
  // Wrap consecutive list items
  html = html.replace(/(<li>.*<\/li>\s*)+/g, (match) => {
    return '<ul>' + match + '</ul>';
  });
  
  // Code blocks
  html = html.replace(/```([^`]+)```/g, '<pre><code>$1</code></pre>');
  
  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
  
  // Blockquotes
  html = html.replace(/^> (.+)$/gim, '<blockquote>$1</blockquote>');
  
  // Horizontal rules
  html = html.replace(/^---$/gim, '<hr>');
  
  // Paragraphs
  html = html.split('\n\n').map(para => {
    // Don't wrap if it's already an HTML element
    if (para.trim().startsWith('<')) {
      return para;
    }
    return para.trim() ? `<p>${para}</p>` : '';
  }).join('\n');
  
  return html;
}

async function loadUpdateInfo() {
  try {
    // Get update info from the query parameters
    const params = new URLSearchParams(window.location.search);
    const updateInfoStr = params.get('updateInfo');
    
    if (updateInfoStr) {
      updateInfo = JSON.parse(decodeURIComponent(updateInfoStr));
    } else {
      // Fallback: check for updates again
      updateInfo = await invoke('check_for_update');
    }
    
    // Update version display
    document.getElementById('current-version').textContent = updateInfo.current_version;
    document.getElementById('new-version').textContent = updateInfo.latest_version;
    
    // Show release notes
    if (updateInfo.release_notes) {
      const notesHtml = parseMarkdown(updateInfo.release_notes);
      document.getElementById('release-notes-content').innerHTML = notesHtml;
      document.getElementById('release-notes').style.display = 'block';
    } else {
      document.getElementById('release-notes-content').innerHTML = '<p>No release notes available.</p>';
      document.getElementById('release-notes').style.display = 'block';
    }
    
    // Hide loading state
    document.getElementById('loading-state').style.display = 'none';
    
  } catch (error) {
    console.error('Failed to load update info:', error);
    document.getElementById('loading-state').style.display = 'none';
    document.getElementById('error-state').style.display = 'block';
  }
}

async function downloadUpdate() {
  if (updateInfo && updateInfo.download_url) {
    try {
      // Use the opener plugin to open URL in default browser
      await invoke('plugin:opener|open_url', { url: updateInfo.download_url });
      
      // Close the update window
      const currentWindow = WebviewWindow.getCurrent();
      await currentWindow.close();
    } catch (error) {
      console.error('Failed to open download URL:', error);
    }
  }
}

async function remindLater() {
  try {
    // Set a timestamp for when to show the update again (24 hours from now)
    const remindTimestamp = Date.now() + (24 * 60 * 60 * 1000); // 24 hours in milliseconds
    await invoke('set_setting', {
      key: 'UpdateRemindAfter',
      value: remindTimestamp.toString()
    });
    
    // Close the update window
    const currentWindow = WebviewWindow.getCurrent();
    await currentWindow.close();
  } catch (error) {
    console.error('Failed to set remind later:', error);
    // Close anyway
    const currentWindow = WebviewWindow.getCurrent();
    await currentWindow.close();
  }
}

// Load update info when the window loads
window.addEventListener('DOMContentLoaded', () => {
  loadUpdateInfo();
});

// Handle keyboard shortcuts
window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    remindLater();
  }
});