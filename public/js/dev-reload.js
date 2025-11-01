// Development live reload functionality
// This script connects to the SSE endpoint and reloads the page when files change

(function () {
    'use strict';

    // Only enable in development mode (when the SSE endpoint is available)
    let retryCount = 0;
    const maxRetries = 3;

    function connectToReloadStream() {
        console.log('[Dev Reload] Attempting to connect to development reload stream...');

        const eventSource = new EventSource('/_dev/reload');

        eventSource.onopen = function (event) {
            console.log('[Dev Reload] Connected to development reload stream');
            retryCount = 0; // Reset retry count on successful connection
        };

        eventSource.onmessage = function (event) {
            console.log('[Dev Reload] Received message:', event.data);
        };

        eventSource.addEventListener('reload', function (event) {
            console.log('[Dev Reload] File change detected, reloading page...');

            // Add a small visual indicator before reload
            const indicator = document.createElement('div');
            indicator.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                background: #10b981;
                color: white;
                padding: 10px 15px;
                border-radius: 5px;
                z-index: 10000;
                font-family: monospace;
                font-size: 14px;
                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                transition: opacity 0.3s ease;
            `;
            indicator.textContent = '🔄 Reloading...';
            document.body.appendChild(indicator);

            // Reload after a short delay to show the indicator
            setTimeout(() => {
                window.location.reload();
            }, 500);
        });

        eventSource.addEventListener('template-changed', function (event) {
            console.log('[Dev Reload] Template change detected - requires recompilation!');

            // Show warning about template changes
            const warning = document.createElement('div');
            warning.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                background: #f59e0b;
                color: white;
                padding: 15px 20px;
                border-radius: 8px;
                z-index: 10000;
                font-family: monospace;
                font-size: 14px;
                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                max-width: 400px;
                line-height: 1.4;
            `;
            warning.innerHTML = `
                <div style="font-weight: bold; margin-bottom: 8px;">⚠️ Template Changed</div>
                <div style="font-size: 12px;">Templates require recompilation.<br>
                Restart with <code>./dev.sh</code> for automatic rebuilds.</div>
            `;
            document.body.appendChild(warning);

            // Auto-remove warning after 8 seconds
            setTimeout(() => {
                if (warning.parentNode) {
                    warning.parentNode.removeChild(warning);
                }
            }, 8000);
        });

        eventSource.onerror = function (event) {
            console.log('[Dev Reload] Connection error or closed');
            eventSource.close();

            // Retry connection with exponential backoff
            if (retryCount < maxRetries) {
                retryCount++;
                const delay = Math.min(1000 * Math.pow(2, retryCount), 10000);
                console.log(`[Dev Reload] Retrying connection in ${delay}ms (attempt ${retryCount}/${maxRetries})`);

                setTimeout(() => {
                    connectToReloadStream();
                }, delay);
            } else {
                console.log('[Dev Reload] Max retries reached, giving up on auto-reload');
            }
        };

        return eventSource;
    }

    // Only attempt connection if we're likely in development mode
    // We can detect this by trying to fetch the reload endpoint
    fetch('/_dev/reload', { method: 'HEAD' })
        .then(response => {
            if (response.ok || response.status === 405) { // HEAD might not be allowed, but GET is
                console.log('[Dev Reload] Development mode detected, enabling live reload');
                connectToReloadStream();
            }
        })
        .catch(() => {
            // Silently fail - probably not in development mode
            console.log('[Dev Reload] Development mode not detected, live reload disabled');
        });
})();