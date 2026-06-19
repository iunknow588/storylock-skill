function ensureUrl(value, fieldName = 'endpoint') {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new Error(`${fieldName} must be a non-empty string`);
  }
  return new URL(value).toString();
}

function ensurePositiveTimeout(value) {
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error('timeoutMs must be a positive number');
  }
  return value;
}

async function parseJsonResponse(response) {
  const text = await response.text();
  return parseJsonText(text, 'remote gateway returned non-JSON response');
}

function parseJsonText(text, errorPrefix) {
  if (!text.trim()) {
    return null;
  }
  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`${errorPrefix}: ${error.message}`);
  }
}

export function createHttpRemoteTransport({
  endpoint,
  fetchImpl = globalThis.fetch,
  headers = {},
  timeoutMs = 15_000,
} = {}) {
  const resolvedEndpoint = ensureUrl(endpoint);
  const resolvedTimeoutMs = ensurePositiveTimeout(timeoutMs);
  if (typeof fetchImpl !== 'function') {
    throw new Error('fetchImpl must be a function');
  }
  return async function httpRemoteTransport(request) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), resolvedTimeoutMs);
    try {
      let response;
      try {
        response = await fetchImpl(resolvedEndpoint, {
          method: 'POST',
          headers: {
            'content-type': 'application/json; charset=utf-8',
            ...headers,
          },
          body: JSON.stringify(request),
          signal: controller.signal,
        });
      } catch (error) {
        if (controller.signal.aborted || error?.name === 'AbortError') {
          throw new Error(`remote gateway request timed out after ${resolvedTimeoutMs}ms`);
        }
        throw error;
      }
      if (!response.ok) {
        const text = await response.text();
        let payload = null;
        if (text.trim()) {
          try {
            payload = parseJsonText(text, 'remote gateway returned invalid error JSON');
          } catch {
            payload = null;
          }
        }
        const message = payload?.message
          ?? payload?.error?.message
          ?? `remote gateway request failed with status ${response.status}`;
        throw new Error(message);
      }
      const payload = await parseJsonResponse(response);
      return payload;
    } finally {
      clearTimeout(timer);
    }
  };
}
