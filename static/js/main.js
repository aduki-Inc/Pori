class Dashboard {
  constructor() {
    this.ws = null;
    this.isConnected = false;
    this.init();
  }

  init() {
    this.setupEventListeners();
    this.connectWebSocket();
    this.startStatsRefresh();
  }

  setupEventListeners() {
    document.getElementById('refresh-btn').addEventListener('click', () => {
      this.refreshStats();
    });

    document.getElementById('clear-logs-btn').addEventListener('click', () => {
      this.clearLogs();
    });
  }

  connectWebSocket() {
    // WebSocket connection for real-time updates
    // This would connect to the dashboard WebSocket endpoint
    console.log('WebSocket connection would be established here');

    // Simulate connection for demo
    setTimeout(() => {
      this.updateConnectionStatus('connected');
    }, 1000);
  }

  updateConnectionStatus(status) {
    const statusElement = document.getElementById('status');
    const statusDot = statusElement.querySelector('.status-dot');
    const statusText = statusElement.querySelector('.status-text');

    switch (status) {
      case 'connected':
        statusDot.classList.add('connected');
        statusText.textContent = 'Connected';
        this.isConnected = true;
        break;
      case 'connecting':
        statusDot.classList.remove('connected');
        statusText.textContent = 'Connecting...';
        this.isConnected = false;
        break;
      case 'disconnected':
        statusDot.classList.remove('connected');
        statusText.textContent = 'Disconnected';
        this.isConnected = false;
        break;
    }
  }

  updateStats(stats) {
    document.getElementById('requests-processed').textContent = stats.requests_processed || 0;
    document.getElementById('requests-successful').textContent = stats.requests_successful || 0;
    document.getElementById('requests-failed').textContent = stats.requests_failed || 0;
    document.getElementById('bytes-transferred').textContent = this.formatBytes(stats.bytes_forwarded || 0);
  }

  addLogEntry(message, type = 'info') {
    const logContainer = document.getElementById('log-container');
    const logEntry = document.createElement('div');
    logEntry.className = `log-entry ${type}`;

    const timestamp = new Date().toLocaleTimeString();
    logEntry.textContent = `[${timestamp}] ${message}`;

    logContainer.appendChild(logEntry);
    logContainer.scrollTop = logContainer.scrollHeight;

    // Keep only last 100 entries
    const entries = logContainer.querySelectorAll('.log-entry');
    if (entries.length > 100) {
      logContainer.removeChild(entries[0]);
    }
  }

  clearLogs() {
    const logContainer = document.getElementById('log-container');
    logContainer.innerHTML = '<div class="log-entry">Logs cleared</div>';
  }

  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  refreshStats() {
    fetch('/api/stats')
      .then(response => response.json())
      .then(stats => {
        this.updateStats(stats);
        this.addLogEntry('Stats refreshed', 'success');
      })
      .catch(error => {
        this.addLogEntry(`Failed to refresh stats: ${error.message}`, 'error');
      });
  }

  startStatsRefresh() {
    // Refresh stats every 5 seconds
    setInterval(() => {
      if (this.isConnected) {
        this.refreshStats();
      }
    }, 5000);
  }
}

// Initialize dashboard when page loads
document.addEventListener('DOMContentLoaded', () => {
  window.dashboard = new Dashboard();
});
