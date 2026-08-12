// Wait for WhatsApp Web to fully load
setTimeout(() => {
    console.log('[WaLinux] Download handler initialized');

    // Track blob URLs created by WhatsApp
    const originalCreateObjectURL = URL.createObjectURL;
    const blobUrls = new Map();

    URL.createObjectURL = function(blob) {
        const url = originalCreateObjectURL.call(this, blob);
        blobUrls.set(url, blob);
        console.log('[WaLinux] Blob created:', url, 'Type:', blob.type, 'Size:', blob.size);
        return url;
    };

    // Intercept clicks on download links/buttons (capture phase to catch early)
    document.addEventListener('click', async (e) => {
        // Find the actual clickable element (could be nested in spans/divs)
        const target = e.target.closest('a, [role="button"], button, [download]');

        if (!target) return;

        console.log('[WaLinux] Click detected on:', target.tagName, target);

        // Check for blob URL in href or data attributes
        const href = target.href || target.getAttribute('href') || target.dataset?.href;

        if (href && href.startsWith('blob:')) {
            console.log('[WaLinux] Blob download intercepted:', href);

            // Prevent default download behavior immediately
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();

            try {
                // Get blob from our tracked map or fetch it
                let blob = blobUrls.get(href);
                if (!blob) {
                    console.log('[WaLinux] Fetching blob from:', href);
                    blob = await fetch(href).then(r => r.blob());
                }

                const arrayBuffer = await blob.arrayBuffer();
                const bytes = Array.from(new Uint8Array(arrayBuffer));

                // Try multiple ways to get the filename
                const filename = target.download ||
                               target.getAttribute('download') ||
                               extractFilename(target) ||
                               generateFilename(blob);

                console.log('[WaLinux] Saving:', filename, 'Size:', bytes.length, 'bytes');

                // Send to Rust backend
                try {
                    const result = await window.__TAURI__.core.invoke('save_blob', {
                        bytes: bytes,
                        filename: filename
                    });

                    console.log('[WaLinux] File saved to:', result);

                    // Show success notification
                    await window.__TAURI__.core.invoke('show_notification', {
                        title: 'Download Complete',
                        body: `Saved ${filename}`,
                        icon: null
                    }).catch(err => console.log('[WaLinux] Notification failed:', err));

                } catch (saveError) {
                    // Check if error is due to file already existing
                    // Rust returns the file path as error when file exists
                    const errorMsg = String(saveError);

                    if (errorMsg.includes('/WaLinux/') || errorMsg.includes(filename)) {
                        // File already exists
                        console.log('[WaLinux] File already exists:', errorMsg);

                        await window.__TAURI__.core.invoke('show_notification', {
                            title: 'Already Downloaded',
                            body: `${filename} already exists`,
                            icon: null
                        }).catch(() => {});
                    } else {
                        // Other save error - re-throw to outer catch
                        throw saveError;
                    }
                }

            } catch (err) {
                console.error('[WaLinux] Failed to save blob:', err);

                // Show error notification
                await window.__TAURI__.core.invoke('show_notification', {
                    title: 'Download Failed',
                    body: `Error: ${err}`,
                    icon: null
                }).catch(() => {});

                // As fallback, let the browser try
                // Remove our listener temporarily and re-click
                // (commented out to avoid loops)
                // location.href = href;
            }
        }
    }, true); // Use capture phase to intercept before other handlers

    // Helper function to extract filename from WhatsApp DOM
    function extractFilename(element) {
        try {
            // Try to find filename in nearby text content
            const parent = element.closest('[data-id], [data-testid], [title]');
            if (parent) {
                // Check title attribute first
                const title = parent.getAttribute('title');
                if (title && /\.\w{2,4}$/.test(title)) {
                    return title;
                }

                // Search text content for filename patterns
                const textContent = parent.textContent || '';
                const filenameMatch = textContent.match(/[\w\s-]+\.(?:jpg|jpeg|png|gif|webp|pdf|mp4|webm|mov|ogg|mp3|wav|m4a|doc|docx|xls|xlsx|ppt|pptx|zip|rar|7z|txt|csv)/i);
                if (filenameMatch) {
                    return filenameMatch[0];
                }
            }

            // Try to find in aria-label
            const ariaLabel = element.getAttribute('aria-label');
            if (ariaLabel && /\.\w{2,4}$/.test(ariaLabel)) {
                return ariaLabel;
            }
        } catch (err) {
            console.log('[WaLinux] Could not extract filename:', err);
        }
        return null;
    }

    // Generate filename based on blob type and timestamp
    function generateFilename(blob) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
        const extensions = {
            'image/jpeg': 'jpg',
            'image/png': 'png',
            'image/gif': 'gif',
            'image/webp': 'webp',
            'video/mp4': 'mp4',
            'video/webm': 'webm',
            'audio/mpeg': 'mp3',
            'audio/ogg': 'ogg',
            'audio/wav': 'wav',
            'application/pdf': 'pdf',
            'application/zip': 'zip',
        };

        const ext = extensions[blob.type] || 'bin';
        return `whatsapp_${timestamp}.${ext}`;
    }

    // Watch for dynamically created download links (WhatsApp often creates temporary <a> elements)
    const observer = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
            for (const node of mutation.addedNodes) {
                if (node.nodeType === Node.ELEMENT_NODE) {
                    // Check if it's a download link
                    if (node.tagName === 'A' && node.href && node.href.startsWith('blob:')) {
                        console.log('[WaLinux] Detected dynamic download link:', node.href, node.download);

                        // Intercept the link before it can be clicked
                        const originalClick = node.click;
                        node.click = async function() {
                            console.log('[WaLinux] Intercepting programmatic click on:', this.href);

                            try {
                                const href = this.href;
                                let blob = blobUrls.get(href);

                                if (!blob) {
                                    console.log('[WaLinux] Fetching blob:', href);
                                    blob = await fetch(href).then(r => r.blob());
                                }

                                const arrayBuffer = await blob.arrayBuffer();
                                const bytes = Array.from(new Uint8Array(arrayBuffer));
                                const filename = this.download || this.getAttribute('download') || generateFilename(blob);

                                console.log('[WaLinux] Saving:', filename);

                                try {
                                    const result = await window.__TAURI__.core.invoke('save_blob', {
                                        bytes: bytes,
                                        filename: filename
                                    });

                                    console.log('[WaLinux] File saved to:', result);

                                    await window.__TAURI__.core.invoke('show_notification', {
                                        title: 'Download Complete',
                                        body: `Saved ${filename}`,
                                        icon: null
                                    }).catch(() => {});

                                } catch (saveError) {
                                    // Check if file already exists
                                    const errorMsg = String(saveError);

                                    if (errorMsg.includes('/WaLinux/') || errorMsg.includes(filename)) {
                                        console.log('[WaLinux] File already exists:', errorMsg);

                                        await window.__TAURI__.core.invoke('show_notification', {
                                            title: 'Already Downloaded',
                                            body: `${filename} already exists`,
                                            icon: null
                                        }).catch(() => {});
                                    } else {
                                        throw saveError;
                                    }
                                }

                                // Don't call original click
                                return;

                            } catch (err) {
                                console.error('[WaLinux] Failed to intercept download:', err);
                                // Fallback to original behavior
                                originalClick.call(this);
                            }
                        };
                    }
                }
            }
        }
    });

    // Start observing the document for added nodes
    observer.observe(document.body, {
        childList: true,
        subtree: true
    });

    console.log('[WaLinux] Download handler ready. Blob tracking active. MutationObserver watching for dynamic downloads.');

}, 3000); // 3-second delay to ensure WhatsApp Web DOM is fully ready
