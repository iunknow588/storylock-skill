function ensureUrl(value, fieldName = 'endpoint') {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return new URL(value).toString();
}

async function parseJsonResponse(response) {
  const text = await response.text();
  if (!text.trim()) {
    return null;
  }
  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`remote gateway returned non-JSON response: ${error.message}`);
  }
}

export function createHttpRemoteTransport({
  endpoint,
  fetchImpl = globalThis.fetch,
  headers = {},
  timeoutMs = 15_000,
} = {}) {
  const resolvedEndpoint = ensureUrl(endpoint);
  if (typeof fetchImpl !== 'function') {
    throw new Error('fetchImpl must be a function');
  }
  return async function httpRemoteTransport(request) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const response = await fetchImpl(resolvedEndpoint, {
        method: 'POST',
        headers: {
          'content-type': 'application/json; charset=utf-8',
          ...headers,
        },
        body: JSON.stringify(request),
        signal: controller.signal,
      });
      const payload = await parseJsonResponse(response);
      if (!response.ok) {
        const message = payload?.message
          ?? payload?.error?.message
          ?? `remote gateway request failed with status ${response.status}`;
        throw new Error(message);
      }
      return payload;
    } finally {
      clearTimeout(timer);
    }
  };
}
