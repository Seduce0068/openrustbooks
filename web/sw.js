/*
 * openrustbooks の最小 Service Worker（REQ-011）。
 *
 * 方針:
 * - 同一起点の GET リクエストだけを扱い、他オリジンは素通しする（fail-closed）。
 * - precache はアプリ shell（index.html / manifest / アイコン）に限定する。
 * - キャッシュ名にバージョンを付け、activate 時に旧キャッシュを削除する。
 * - インストール・登録に失敗しても通常の Web アプリとして動き続ける。
 */
const CACHE_PREFIX = "orb-shell-";
const CACHE_NAME = `${CACHE_PREFIX}v1`;
const SHELL_URLS = [
  "/",
  "/index.html",
  "/manifest.webmanifest",
  "/icons/app-192.png",
  "/icons/app-512.png",
  "/icons/maskable-192.png",
  "/icons/maskable-512.png",
  "/icons/apple-touch-icon.png",
  "/favicon.svg",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);
      // 1 つでも取得できなければ shell を完全に揃えられないため、install を失敗させる。
      // 部分キャッシュで不整合なオフライン画面を出さない（fail-closed）。
      await cache.addAll(SHELL_URLS);
      await self.skipWaiting();
    })(),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys
          .filter((key) => key.startsWith(CACHE_PREFIX) && key !== CACHE_NAME)
          .map((key) => caches.delete(key)),
      );
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") {
    return;
  }
  let url;
  try {
    url = new URL(request.url);
  } catch {
    return;
  }
  if (url.origin !== self.location.origin) {
    return;
  }

  if (request.mode === "navigate") {
    // ナビゲーションは network-first。オフライン時だけ shell を返す。
    event.respondWith(
      fetch(request).catch(async () => {
        const cache = await caches.open(CACHE_NAME);
        return (await cache.match("/index.html")) || Response.error();
      }),
    );
    return;
  }

  // 静的アセットは cache-first。無ければネットワークへ委譲する。
  event.respondWith(
    caches.match(request).then((cached) => {
      if (cached) {
        return cached;
      }
      return fetch(request).then((response) => {
        if (response.ok && response.type === "basic") {
          const copy = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(request, copy));
        }
        return response;
      });
    }),
  );
});
