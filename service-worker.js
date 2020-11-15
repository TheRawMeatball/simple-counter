const CACHE = 'precache-v1.5';

const PRECACHE_URLS = [
    '/index.html',
    '/',
    '/res/style.css',
    '/res/favicon-16x16.png',
    '/res/favicon-32x32.png',
    '/res/favicon.ico',
    '/res/apple-touch-icon.png',
    '/res/android-chrome-192x192.png',
    '/res/android-chrome-512x512.png',
    '/pkg/package_bg.wasm',
    '/pkg/package.js',
];

self.addEventListener('install', event => {
    console.log("installing");
    event.waitUntil(
        caches.open(CACHE)
            .then(cache => cache.addAll(PRECACHE_URLS))
            .then(self.skipWaiting())
    );
});

self.addEventListener('activate', event => {
    console.log("active");
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return cacheNames.filter(cacheName => CACHE != cacheName);
        }).then(cachesToDelete => {
            return Promise.all(cachesToDelete.map(cacheToDelete => {
                return caches.delete(cacheToDelete);
            }));
        }).then(() => self.clients.claim())
    );
});

self.addEventListener('fetch', event => {
    if (event.request.url.startsWith(self.location.origin)) {
        event.respondWith((async () => {
            try {
                let cachedResponse = await caches.match(event.request);
                if (cachedResponse) {
                    return cachedResponse;
                } else {
                    return await caches.match('/');
                }
            } catch (e) {
                console.log("failed to send cached file");
                console.log(e);
            }
        })());
        (async () => {
            try {
                let upToDate = await fetch(event.request);
                let cache = await caches.open(CACHE);
                cache.put(event.request, upToDate);
            } catch (e) {
                console.log(`failed to update cache, ${e}`);
            }
        })();
    }
});