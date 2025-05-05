// @ts-check
/// <reference lib="dom" />

/**
 * @import { CachedFaviconRequest, CachedFaviconResponse } from "./index.d.js"
 */

/**
 * Bang wrapper class
 * @class
 */
class Bang {
  /**
   * @param {string} s - The short
   * @param {string} t - The trigger
   * @param {string} u - The url template
   */
  constructor(s, t, u) {
    /** @type {string} */
    this.short = s;

    /** @type {string} */
    this.trigger = t;

    /** @type {string} */
    this.url_template = u;
  }

  /**
   * @param {string} str
   */
  query(str) {
    return [this.short, this.trigger, this.url_template].some((prop) =>
      prop.toString().toLowerCase().includes(str),
    );
  }
}

if ("serviceWorker" in navigator) {
  navigator.serviceWorker
    .register("/sw.js")
    .then((registration) => {
      console.log("Service Worker registered.");

      // Listen for updates
      registration.addEventListener("updatefound", () => {
        const newWorker = registration.installing;
        console.log("New service worker found:", newWorker);

        newWorker?.addEventListener("statechange", () => {
          if (
            newWorker.state === "installed" &&
            navigator.serviceWorker.controller
          ) {
            // New update is ready
            console.log("New content is available!");

            // Check if the user is interacting with the page
            if (document.hasFocus() && !isUserInteracting()) {
              console.log("Refreshing in the background...");
              window.location.reload(); // Refresh when safe
            } else {
              // If the user is active, don't force the refresh yet
              console.log("Waiting for user to idle before refresh.");
            }
          }
        });
      });
    })
    .catch((err) => {
      console.error("Service Worker registration failed:", err);
    });
}

// Function to detect if the user is interacting with the page
function isUserInteracting() {
  let isInteracting = false;

  // Detect user activity (click, scroll, typing)
  const userEvents = ["click", "keydown", "mousemove", "scroll"];

  const handleUserActivity = () => {
    isInteracting = true;
    console.log("User is interacting with the page.");
    setTimeout(() => (isInteracting = false), 5000); // Reset interaction after 5 seconds of inactivity
  };

  userEvents.forEach((event) => {
    window.addEventListener(event, handleUserActivity, { once: true });
  });

  return isInteracting;
}

let raw = document.getElementById("bang-data")?.textContent ?? "{}";

/**
 * @type {Bang[]}
 */
const bangs = JSON.parse(raw).map(
  (/** @type {{s: string, t: string, u: string }} */ b) =>
    new Bang(b.s, b.t, b.u),
);

const bang_len = bangs.length;
const pagination = {
  max_items: 50,
  active_page_index: getPageFromURL(),
  page_count: Math.ceil(bang_len / 50),
};

let bang_container;

const _urlQuery = getQueryFromURL();

/** @type {((bang: Bang) => boolean) | null} */
let active_filter_fn =
  _urlQuery != null ? (bang) => bang.query(_urlQuery) : null;

window.onload = () => {
  /// @ts-ignore
  raw = undefined;

  bang_container = document.querySelector("table#bangs tbody");
  loadBangs(active_filter_fn);

  document.getElementById("next")?.addEventListener("click", () => {
    if (pagination.active_page_index < pagination.page_count - 1) {
      goToPage(pagination.active_page_index + 1);
    }
  }) ?? console.warn("Could not add event listener to #next");

  document.getElementById("prev")?.addEventListener("click", () => {
    if (pagination.active_page_index > 0) {
      goToPage(pagination.active_page_index - 1);
    }
  }) ?? console.warn("Could not add event listener to #prev");

  /**
   * @type {HTMLInputElement|undefined}
   */
  const input = /** @type {HTMLInputElement|undefined}*/ (
    document.querySelector("input[name='bang-search']")
  );

  _urlQuery != null && input && (input.value = _urlQuery);

  input?.addEventListener("input", (e) => {
    const target = /** @type {HTMLInputElement} */ (e.currentTarget);

    setTimeout(() => persistQueryToURL(target.value), 0);

    if (target.value?.trim()?.length !== 0) {
      pagination.active_page_index = 0;
      active_filter_fn = (bang, ..._) => bang.query(target.value.toLowerCase());
      loadBangs(active_filter_fn);
    } else {
      if (active_filter_fn === null) return;

      pagination.active_page_index = 0;
      active_filter_fn = null;
      loadBangs(active_filter_fn);
    }
  }) ?? console.warn("Could not add event listener to search input");
};

window.onpopstate =
  /**
   *@param {PopStateEvent} event
   */
  (event) => {
    if (event.state && typeof event.state.pageIndex === "number") {
      pagination.active_page_index = event.state.pageIndex;
      loadBangs(active_filter_fn);
    }
  };

/**
 * @returns {number}
 */
function getPageFromURL() {
  const params = new URLSearchParams(window.location.search);
  const page = parseInt(params.get("page") || "0", 10);
  return isNaN(page) || page < 0 ? 0 : page;
}

/**
 * @param {number} pageIndex
 * @param {boolean} refresh - Refresh the bangs/only change the history
 */
function goToPage(pageIndex, refresh = true) {
  pagination.active_page_index = pageIndex;

  const params = new URLSearchParams(window.location.search);
  params.set("page", pageIndex.toString());

  // Push new state to history WITHOUT reloading
  history.pushState({ pageIndex }, "", "?" + params.toString());

  // Load new page data
  refresh && loadBangs(active_filter_fn);
}

/**
 * @returns {string | null}
 */
function getQueryFromURL() {
  const params = new URLSearchParams(window.location.search);
  const query = params.get("query");
  return query?.trim()?.toLowerCase() ?? null;
}

/**
 * @param {string} query
 */
function persistQueryToURL(query) {
  const params = new URLSearchParams(window.location.search);
  params.set("query", query);

  // Push new state to history WITHOUT reloading
  history.pushState({ query }, "", "?" + params.toString());
}

/**
 * @param {(function(Bang, ...any): boolean) | null} filter
 */
function loadBangs(filter) {
  const _bangs = filter === null ? bangs : bangs.filter(filter);
  const start = pagination.active_page_index * pagination.max_items;
  const end = start + pagination.max_items;
  const currentItems = _bangs.slice(start, end);

  pagination.page_count = Math.max(
    Math.ceil(_bangs.length / pagination.max_items),
    1,
  );

  goToPage(
    Math.max(
      Math.min(pagination.active_page_index, pagination.page_count - 1),
      0,
    ),
    false,
  );

  bang_container.innerHTML = "";

  for (const bang of currentItems) {
    bang_container.appendChild(buildBangElement(bang));
  }

  const pageInfo = document.getElementById("page-info");
  if (pageInfo) {
    pageInfo.textContent = `Page ${pagination.active_page_index + 1} of ${pagination.page_count}`;
  } else
    console.warn(
      "Could not set textContent for #page-info. Element not found.",
    );
}

/**
 * Builds a HTML Element resembling:
 * <tr>
 *  <td class="image">
 *    <div class="image-container">
 *      <img src="..." data-fallback="<0 | 1>" ...>
 *    </div>
 *    <span>
 *    <!-- Bang Short Name -->
 *    </span>
 *  </td>
 *  <td>
 *  <!-- Bang Trigger -->
 *  </td>
 *  <td>
 *    <a href="<bang url template domain>" target="_blank">
 *    <!-- Bang URL Template -->
 *    </a>
 *  </td>
 * </tr>
 *
 * @param {Bang} bang
 * @returns {HTMLTableRowElement}
 */
function buildBangElement(bang) {
  const row = document.createElement("tr");

  const imageContainer = document.createElement("div");
  imageContainer.className = "image-container";

  const faviconCell = document.createElement("img");
  faviconCell.src = "/assets/bangs/fallback-icon.svg";
  faviconCell.dataset["fallback"] = "1";
  faviconCell.decoding = "async";

  const shortCell = document.createElement("td");
  shortCell.className = "image";

  const shortText = document.createElement("span");
  shortText.textContent = bang.short;

  imageContainer.appendChild(faviconCell);

  shortCell.appendChild(imageContainer);
  shortCell.appendChild(shortText);

  const triggerCell = document.createElement("td");
  triggerCell.textContent = bang.trigger;

  const templateCell = document.createElement("td");
  const link = document.createElement("a");

  /**
   * @type URL
   */
  let url;
  try {
    url = new URL(bang.url_template);
    link.href = url.origin;
  } catch (_) {
    url = new URL(location.href + "/assets/bangs/fallback-icon.svg");
    link.href = "#";
  }

  link.textContent = bang.url_template.toString();
  link.target = "_blank";
  templateCell.appendChild(link);

  row.appendChild(shortCell);
  row.appendChild(triggerCell);
  row.appendChild(templateCell);

  setFavicon(shortCell, url);
  return row;
}

/**
 * @param {string} url
 * @returns {Promise<boolean>}
 */
function isFaviconCached(url) {
  return new Promise((resolve) => {
    if (!navigator.serviceWorker.controller) {
      console.debug("Returning early: controller wasn't found.");
      resolve(false);
      return;
    }

    const channel = new MessageChannel();
    channel.port1.onmessage = (event) => {
      /**
       * @type CachedFaviconResponse
       */
      const data = event.data;
      console.assert(data.message == "IS_FAVICON_CACHED");
      resolve(data.response.isCached);
    };

    /**
     * @type CachedFaviconRequest
     */
    const request = {
      message: "IS_FAVICON_CACHED",
      request: { url },
    };
    navigator.serviceWorker.controller.postMessage(request, [channel.port2]);
  });
}

/**
 * Sets the src of the img element within the <td> (in place)
 * Attempts to cache and/or use a cached favicon, fetching it when required.
 * @param {HTMLTableCellElement} el - The <td> element encapsulating the favicons
 * @param {URL} url
 *
 * @returns {Promise<void>}
 */
async function setFavicon(el, url) {
  const img = el.querySelector("img");
  if (!img) return;

  const originFavicon = `${url.origin}/favicon.ico`;

  const cached = await isFaviconCached(originFavicon);

  /**
   * @type HTMLTableCellElement | null
   */
  const imageContainer = el.querySelector(".image-container");
  if (!imageContainer) return console.warn("Could not select .image-container");

  crossfadeImage(imageContainer, img, originFavicon, cached ? 0 : 350).then(
    (newImg) => {
      try {
        new URL(newImg.src).origin != window.origin &&
          (newImg.dataset["fallback"] = "0");
      } catch {}
    },
  );
}

/**
 * @param {HTMLTableCellElement} wrapper
 * @param {HTMLImageElement} imageEl
 * @param {string} nextUrl
 * @param {number} duration
 * @returns {Promise<HTMLImageElement>}
 */
function crossfadeImage(wrapper, imageEl, nextUrl, duration = 1000) {
  const next = document.createElement("img");
  next.src = nextUrl;

  Object.assign(next.style, {
    opacity: "0",
    transition: `opacity ${duration}ms ease-out`,
    pointerEvents: "none",
  });

  wrapper.append(next);

  next.onload = () => {
    // Force DOM recalc
    void next.offsetWidth;

    imageEl.style.opacity = "0";
    next.style.opacity = "1";
  };

  let hasErrored = false;
  next.onerror = () => {
    if (hasErrored) return;
    hasErrored = true;

    next.src = `https://icons.duckduckgo.com/ip3/${new URL(nextUrl).hostname}.ico`;
  };

  return new Promise((resolve) => {
    next.addEventListener(
      "transitionend",
      (_event) => {
        imageEl.remove();
        resolve(next);
      },
      { once: true },
    );
  });
}
