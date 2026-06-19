import { randomUUID } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

function optionalString(value) {
  if (typeof value !== 'string') {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function asUrl(value) {
  const text = optionalString(value);
  if (!text) {
    return null;
  }
  try {
    return new URL(text).toString();
  } catch {
    return text;
  }
}

function hostIdFor(deviceId, appInstanceId) {
  return `${deviceId}:${appInstanceId}`;
}

function compareByLastSeen(a, b) {
  return (b.lastSeenAt ?? 0) - (a.lastSeenAt ?? 0);
}

export class StoryLockHostRegistry {
  constructor({
    filePath = null,
    hostTtlMs = 120_000,
    hostRetentionMs = 24 * 60 * 60_000,
    bindingTokenTtlMs = 10 * 60_000,
    relayTimeoutMs = 35_000,
    relayPollIntervalMs = 2_000,
    relayRetryBackoffMs = [500, 1_500, 5_000],
    now = () => Date.now(),
  } = {}) {
    this.filePath = optionalString(filePath);
    this.hostTtlMs = hostTtlMs;
    this.hostRetentionMs = hostRetentionMs;
    this.bindingTokenTtlMs = bindingTokenTtlMs;
    this.relayTimeoutMs = relayTimeoutMs;
    this.relayPollIntervalMs = relayPollIntervalMs;
    this.relayRetryBackoffMs = Array.isArray(relayRetryBackoffMs)
      ? relayRetryBackoffMs.filter((value) => Number.isFinite(value) && value >= 0)
      : [];
    this.now = now;
    this.hosts = new Map();
    this.bindingTokens = new Map();
    this.relayQueues = new Map();
    this.pendingRelayResponses = new Map();
    this.relayStats = {
      totalRequests: 0,
      resolvedResponses: 0,
      timeoutCount: 0,
      lastTimeoutAt: null,
      lastResolvedAt: null,
    };
    this.load();
  }

  load() {
    if (!this.filePath || !existsSync(this.filePath)) {
      return;
    }
    const text = readFileSync(this.filePath, 'utf8');
    const parsed = JSON.parse(text);
    const hosts = Array.isArray(parsed?.hosts) ? parsed.hosts : [];
    for (const item of hosts) {
      const normalized = this.normalizeHostRecord(item, item.updatedAt ?? this.now());
      this.hosts.set(normalized.hostId, normalized);
    }
    this.pruneExpiredHosts();
  }

  persist() {
    if (!this.filePath) {
      return;
    }
    mkdirSync(dirname(this.filePath), { recursive: true });
    writeFileSync(this.filePath, JSON.stringify({
      hosts: Array.from(this.hosts.values()),
      updatedAt: this.now(),
    }, null, 2));
  }

  normalizeHostRecord(input, now = this.now()) {
    const deviceId = optionalString(input.deviceId);
    const appInstanceId = optionalString(input.appInstanceId);
    if (!deviceId || !appInstanceId) {
      throw new Error('deviceId and appInstanceId are required');
    }
    const createdAt = Number.isFinite(input.createdAt) ? input.createdAt : now;
    const updatedAt = Number.isFinite(input.updatedAt) ? input.updatedAt : now;
    const lastSeenAt = Number.isFinite(input.lastSeenAt) ? input.lastSeenAt : now;
    return {
      hostId: hostIdFor(deviceId, appInstanceId),
      registrationId: optionalString(input.registrationId) ?? `reg-${randomUUID()}`,
      deviceId,
      appInstanceId,
      identityId: optionalString(input.identityId) ?? 'unknown-identity',
      preferredMode: optionalString(input.preferredMode) ?? 'relay_url',
      directUrl: asUrl(input.directUrl),
      healthUrl: asUrl(input.healthUrl),
      executeUrl: asUrl(input.executeUrl),
      deepLink: optionalString(input.deepLink),
      relayUrl: asUrl(input.relayUrl),
      install: input.install && typeof input.install === 'object' ? { ...input.install } : {},
      device: input.device && typeof input.device === 'object' ? { ...input.device } : {},
      reachability: input.reachability && typeof input.reachability === 'object' ? { ...input.reachability } : {},
      createdAt,
      updatedAt,
      lastSeenAt,
      status: optionalString(input.status) ?? 'active',
      lastError: optionalString(input.lastError),
      bindingTokenId: optionalString(input.bindingTokenId),
    };
  }

  upsertHost(input) {
    this.cleanupBindingTokens();
    this.pruneExpiredHosts();
    const now = this.now();
    const record = this.normalizeHostRecord({
      ...input,
      updatedAt: now,
      lastSeenAt: now,
    }, now);
    const previous = this.hosts.get(record.hostId);
    const merged = previous
      ? {
        ...previous,
        ...record,
        createdAt: previous.createdAt,
      }
      : record;
    this.hosts.set(merged.hostId, merged);
    this.persist();
    return this.presentHost(merged);
  }

  touchHost(deviceId, appInstanceId, patch = {}) {
    this.pruneExpiredHosts();
    const hostId = hostIdFor(deviceId, appInstanceId);
    const existing = this.hosts.get(hostId);
    if (!existing) {
      return null;
    }
    const next = this.normalizeHostRecord({
      ...existing,
      ...patch,
      deviceId,
      appInstanceId,
      updatedAt: this.now(),
      lastSeenAt: this.now(),
    });
    this.hosts.set(hostId, next);
    this.persist();
    return this.presentHost(next);
  }

  listHosts({ identityId = null } = {}) {
    this.cleanupBindingTokens();
    this.pruneExpiredHosts();
    const now = this.now();
    return Array.from(this.hosts.values())
      .filter((host) => !identityId || host.identityId === identityId)
      .sort(compareByLastSeen)
      .map((host) => this.presentHost(host, now));
  }

  presentHost(host, now = this.now()) {
    const ageMs = Math.max(0, now - (host.lastSeenAt ?? 0));
    return {
      ...host,
      online: ageMs <= this.hostTtlMs,
      ageMs,
    };
  }

  issueBindingToken({
    identityId,
    preferredMode = 'relay_url',
    gatewayBaseUrl,
    deepLinkBase = null,
    ttlMs = this.bindingTokenTtlMs,
  } = {}) {
    this.cleanupBindingTokens();
    const token = `slt-${randomUUID()}`;
    const now = this.now();
    const expiresAt = now + ttlMs;
    const record = {
      token,
      tokenId: `bind-${randomUUID()}`,
      identityId: optionalString(identityId) ?? 'android-demo-001',
      preferredMode: optionalString(preferredMode) ?? 'relay_url',
      gatewayBaseUrl: optionalString(gatewayBaseUrl),
      deepLinkBase: optionalString(deepLinkBase),
      expiresAt,
      issuedAt: now,
      consumedAt: null,
    };
    this.bindingTokens.set(token, record);
    return {
      ...record,
      deepLink: this.buildDeepLink(record),
    };
  }

  buildDeepLink(record) {
    const base = optionalString(record.deepLinkBase) ?? 'storylock-host://bind';
    const url = new URL(base);
    if (record.gatewayBaseUrl) {
      url.searchParams.set('gateway_url', record.gatewayBaseUrl);
    }
    url.searchParams.set('binding_token', record.token);
    url.searchParams.set('identity_id', record.identityId);
    url.searchParams.set('preferred_mode', record.preferredMode);
    return url.toString();
  }

  consumeBindingToken(token) {
    this.cleanupBindingTokens();
    const record = this.bindingTokens.get(token);
    if (!record) {
      return null;
    }
    if (record.consumedAt || record.expiresAt <= this.now()) {
      this.bindingTokens.delete(token);
      return null;
    }
    const consumed = {
      ...record,
      consumedAt: this.now(),
    };
    this.bindingTokens.delete(token);
    return { ...consumed, deepLink: this.buildDeepLink(consumed) };
  }

  findHostForInvocation({
    identityId,
    preferredMode = 'relay_url',
  } = {}) {
    const candidates = this.listHosts({ identityId })
      .filter((host) => host.online && host.status === 'active');
    const scored = candidates
      .map((host) => ({
        host,
        score: this.scoreHostForInvocation(host, preferredMode),
      }))
      .sort((left, right) => right.score - left.score || compareByLastSeen(left.host, right.host));
    return scored[0]?.host ?? null;
  }

  createRelayRequest({ hostId, request, timeoutMs = this.relayTimeoutMs }) {
    this.pruneExpiredHosts();
    const host = this.hosts.get(hostId);
    if (!host) {
      throw new Error('registered host not found');
    }
    const relayRequestId = `relay-${randomUUID()}`;
    const queueItem = {
      relayRequestId,
      hostId,
      request,
      createdAt: this.now(),
    };
    const queue = this.relayQueues.get(hostId) ?? [];
    queue.push(queueItem);
    this.relayQueues.set(hostId, queue);

    const response = new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pendingRelayResponses.delete(relayRequestId);
        this.relayStats.timeoutCount += 1;
        this.relayStats.lastTimeoutAt = this.now();
        this.touchHost(host.deviceId, host.appInstanceId, {
          lastError: 'android relay response timeout',
        });
        reject(new Error('android relay response timeout'));
      }, timeoutMs);
      this.pendingRelayResponses.set(relayRequestId, {
        resolve(value) {
          clearTimeout(timer);
          resolve(value);
        },
        reject(error) {
          clearTimeout(timer);
          reject(error);
        },
      });
    });
    this.relayStats.totalRequests += 1;

    return {
      relayRequestId,
      response,
    };
  }

  takeRelayRequest({ deviceId, appInstanceId }) {
    this.pruneExpiredHosts();
    const hostId = hostIdFor(deviceId, appInstanceId);
    const queue = this.relayQueues.get(hostId) ?? [];
    const item = queue.shift() ?? null;
    this.relayQueues.set(hostId, queue);
    if (!item) {
      this.touchHost(deviceId, appInstanceId);
      return null;
    }
    this.touchHost(deviceId, appInstanceId);
    return {
      ...item,
      status: 'dispatch',
    };
  }

  resolveRelayResponse({ relayRequestId, response }) {
    const pending = this.pendingRelayResponses.get(relayRequestId);
    if (!pending) {
      return false;
    }
    this.pendingRelayResponses.delete(relayRequestId);
    this.relayStats.resolvedResponses += 1;
    this.relayStats.lastResolvedAt = this.now();
    pending.resolve(response);
    return true;
  }

  cleanupBindingTokens(now = this.now()) {
    for (const [token, record] of this.bindingTokens.entries()) {
      if ((record.expiresAt ?? 0) <= now || record.consumedAt) {
        this.bindingTokens.delete(token);
      }
    }
  }

  pruneExpiredHosts(now = this.now()) {
    let changed = false;
    for (const [hostId, host] of this.hosts.entries()) {
      const ageMs = Math.max(0, now - (host.lastSeenAt ?? 0));
      if (ageMs > this.hostRetentionMs || host.status === 'revoked') {
        this.hosts.delete(hostId);
        this.relayQueues.delete(hostId);
        changed = true;
      }
    }
    if (changed) {
      this.persist();
    }
  }

  scoreHostForInvocation(host, preferredMode) {
    let score = host.online ? 10_000 : 0;
    if (host.preferredMode === preferredMode) {
      score += 1_000;
    }
    if (preferredMode === 'direct_url' && host.executeUrl) {
      score += 200;
    }
    if (preferredMode === 'relay_url' && host.relayUrl) {
      score += 200;
    }
    score += Math.max(0, this.hostTtlMs - Math.min(host.ageMs ?? this.hostTtlMs, this.hostTtlMs));
    return score;
  }

  getStatusSummary() {
    const hosts = this.listHosts();
    const onlineHosts = hosts.filter((host) => host.online);
    return {
      activeHostCount: onlineHosts.length,
      onlineHostCount: onlineHosts.length,
      totalHostCount: hosts.length,
      hostTtlMs: this.hostTtlMs,
      hostRetentionMs: this.hostRetentionMs,
      cleanupPolicy: {
        ttlMs: this.hostTtlMs,
        retentionMs: this.hostRetentionMs,
        onlineRule: 'last_seen_within_ttl',
        evictionRule: 'remove_after_retention_or_revocation',
      },
      bindingTokenPolicy: {
        ttlMs: this.bindingTokenTtlMs,
        issueRule: 'issue_new_token_per_bind_request',
        consumeRule: 'single_use',
        rotationTrigger: 'expire_or_consume_then_reissue',
      },
      relay: {
        timeoutMs: this.relayTimeoutMs,
        pollIntervalMs: this.relayPollIntervalMs,
        retryBackoffMs: this.relayRetryBackoffMs,
        pendingResponseCount: this.pendingRelayResponses.size,
        totalRequests: this.relayStats.totalRequests,
        resolvedResponses: this.relayStats.resolvedResponses,
        timeoutCount: this.relayStats.timeoutCount,
        lastTimeoutAt: this.relayStats.lastTimeoutAt,
        lastResolvedAt: this.relayStats.lastResolvedAt,
      },
      hosts,
    };
  }
}
