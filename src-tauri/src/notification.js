setTimeout(() => {
    window.Notification = function(title, options) {
        if (window.__TAURI__?.core) {
            window.__TAURI__.core.invoke('show_notification', {
                title: title,
                body: options?.body || '',
                icon: options?.icon || null
            }).catch(() => {});
        }

        const mockNotification = {
            title: title,
            body: options?.body || '',
            icon: options?.icon || '',
            tag: options?.tag || '',
            data: options?.data || null,
            close: function() { if (this.onclose) this.onclose(); },
            addEventListener: function(event, callback) { this['on'+event] = callback; },
            removeEventListener: function(event) { this['on'+event] = null; },
            onclick: null,
            onclose: null,
            onerror: null,
            onshow: null
        };

        setTimeout(() => { if (mockNotification.onshow) mockNotification.onshow(); }, 0);
        return mockNotification;
    };

    Object.defineProperty(window.Notification, 'permission', { get: () => 'granted', enumerable: true, configurable: true });
    window.Notification.requestPermission = async () => 'granted';
}, 3000);
