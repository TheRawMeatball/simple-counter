const CACHE = 'precache-v1.3';

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
    console.log("request");
    if (event.request.url.startsWith(self.location.origin)) {
        event.respondWith(
            caches.match(event.request).then(cachedResponse => {
                if (cachedResponse) {
                    return cachedResponse;
                } else {
                    return caches.match('/');
                }
            })
        );
    }
});