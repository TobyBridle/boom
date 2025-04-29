type CachedFaviconKey = "IS_FAVICON_CACHED";

export type CachedFaviconRequest = {
  message: CachedFaviconKey;
  request: {
    url: string;
  };
};

export type CachedFaviconResponse = {
  message: CachedFaviconKey;
  response: {
    isCached: boolean;
  };
};
