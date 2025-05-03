// @ts-check
/// <reference lib="webworker" />
/// <reference lib="webworker.iterable" />

/**
 * @import { CachedFaviconRequest, CachedFaviconResponse } from "./index.d.js"
 */

/** @type {ServiceWorkerGlobalScope} */
// @ts-ignore
const selfTyped = /** @type {ServiceWorkerGlobalScope} */ (self);

const CACHE_VERSION = 2;
const CACHE_NAME = `favicons-v${CACHE_VERSION}`;

/**
 * @type {(resources: Array<string>) => Promise<void>}
 */
const addResourcesToCache = async (resources) => {
  const cache = await caches.open(CACHE_NAME);
  try {
    await cache.addAll(resources);
  } catch (error) {
    console.error("Cache addAll failed, trying to add individually", error);

    // Try adding one-by-one if addAll fails
    await Promise.all(
      resources.map(async (resource) => {
        try {
          const response = await fetch(resource, { mode: "no-cors" });
          if (response && (response.ok || response.type === "opaque")) {
            await cache.put(resource, response);
          } else {
            console.warn(
              "Not caching resource:",
              resource,
              "Status:",
              response.status,
            );
          }
        } catch (err) {
          console.warn("Failed to cache resource:", resource, err);
        }
      }),
    );
  }
};

/**
 * @type {(req: RequestInfo | URL, res: Response) => Promise<void>}
 */
const addResourceToCache = async (req, res) => {
  try {
    const cache = await caches.open(CACHE_NAME);
    await cache.put(req, res);
  } catch (err) {
    console.error("Failed to add resource to cache.", err);
  }
};

/**
 * Fetch with optional preload and fallback handling.
 *
 * @param {Object} options
 * @param {Request} options.request - The request to fetch.
 * @param {Promise<Response>|undefined} options.preloadResponsePromise - A preload response promise, if available.
 * @param {string} options.fallbackUrl - A fallback URL to use if the request fails.
 * @returns {Promise<Response>} The fetched response.
 */
const fetchWithCache = async ({
  request,
  preloadResponsePromise,
  fallbackUrl,
}) => {
  const clonedRequest = request.clone();

  const cachedResponse = await caches.match(clonedRequest);
  if (cachedResponse) {
    return cachedResponse;
  }

  const preloadResponse = preloadResponsePromise
    ? await preloadResponsePromise
    : undefined;
  if (preloadResponse) {
    addResourceToCache(request, preloadResponse.clone());
    return preloadResponse;
  }

  try {
    const response = await fetch(clonedRequest, { mode: "no-cors" });
    addResourceToCache(request, response.clone());
    return response;
  } catch (e) {
    const fallbackResponse = await caches.match(fallbackUrl);
    if (fallbackResponse) {
      return fallbackResponse;
    }

    console.warn("The fallback `", fallbackUrl, "` is not within the cache.");
    return new Response("Network error occurred", {
      status: 400,
      headers: { "Content-Type": "text/plain" },
    });
  }
};

const enableNavigationPreload = async () => {
  if (selfTyped.registration.navigationPreload) {
    await selfTyped.registration.navigationPreload.enable();
  }
};

selfTyped.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      addResourcesToCache([
        "/assets/bangs/index.js",
        "/assets/bangs/index.html",
        "/assets/bangs/style.css",
        "/assets/bangs/fallback-icon.svg",
      ]).then((_) => console.log("Added fallbacks to cache"));
      selfTyped.skipWaiting();
    })(),
  );
});

selfTyped.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const cacheNames = await caches.keys();
      for (const cacheName of cacheNames) {
        if (cacheName !== CACHE_NAME) {
          console.log("Purging invalid cache:", cacheName);
          await caches.delete(cacheName);
        }
      }
      await enableNavigationPreload();
      await selfTyped.clients.claim();
    })(),
  );
});

selfTyped.addEventListener("fetch", (event) => {
  event.respondWith(
    (async () => {
      return fetchWithCache({
        request: event.request,
        preloadResponsePromise: event.preloadResponse,
        fallbackUrl: event.request.url.endsWith(".ico")
          ? "/assets/bangs/fallback-icon.svg"
          : undefined,
      });
    })(),
  );
});

selfTyped.addEventListener("message", async (event) => {
  const responsePort = event.ports[0];

  switch (event.data.message.trim()) {
    case "IS_FAVICON_CACHED":
      /**
       * @type CachedFaviconRequest
       */
      const data = event.data;

      let cache = await caches.open(CACHE_NAME);
      let isCached = (await cache.keys()).some(
        (key) => key.url === data.request.url,
      );

      /**
       * @type CachedFaviconResponse
       */
      let response = {
        message: "IS_FAVICON_CACHED",
        response: { isCached },
      };

      responsePort.postMessage(response, {});
      break;

    default:
      console.error(
        "Message with type '",
        event.data.type,
        "' received but cannot be handled.",
      );
      break;
  }
});
