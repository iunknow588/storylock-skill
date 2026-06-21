export const AUTHORIZATION_CHANNELS = Object.freeze({
  SINGLE_READ: 'single_read',
  BATCH_READ: 'batch_read',
  STORY_EDIT: 'story_edit',
});

export const AUTHORIZATION_CHANNEL_POLICY = Object.freeze({
  [AUTHORIZATION_CHANNELS.SINGLE_READ]: Object.freeze({
    channel: AUTHORIZATION_CHANNELS.SINGLE_READ,
    storyNodeThreshold: 6,
    requiredStrength: 'medium',
    gridPolicy: Object.freeze({ gridSize: 9, requiredCells: 6 }),
    remoteAllowed: true,
    localOnly: false,
  }),
  [AUTHORIZATION_CHANNELS.BATCH_READ]: Object.freeze({
    channel: AUTHORIZATION_CHANNELS.BATCH_READ,
    storyNodeThreshold: 12,
    requiredStrength: 'high',
    gridPolicy: Object.freeze({ gridSize: 12, requiredCells: 12 }),
    remoteAllowed: true,
    localOnly: false,
  }),
  [AUTHORIZATION_CHANNELS.STORY_EDIT]: Object.freeze({
    channel: AUTHORIZATION_CHANNELS.STORY_EDIT,
    storyNodeThreshold: 22,
    requiredStrength: 'story_edit',
    gridPolicy: Object.freeze({ gridSize: 24, requiredCells: 22 }),
    remoteAllowed: false,
    localOnly: true,
  }),
});

export function normalizeAuthorizationChannel(value, fallback = AUTHORIZATION_CHANNELS.SINGLE_READ) {
  const channel = typeof value === 'string' && value.trim() ? value.trim() : fallback;
  if (!Object.hasOwn(AUTHORIZATION_CHANNEL_POLICY, channel)) {
    throw new Error('authorizationChannel must be single_read, batch_read, or story_edit');
  }
  return channel;
}

export function channelPolicyFor(value) {
  return AUTHORIZATION_CHANNEL_POLICY[normalizeAuthorizationChannel(value)];
}

export function resolveAuthorizationChannel({ objectType, requestedAction, policyHints = {} }) {
  if (policyHints.authorizationChannel) {
    return normalizeAuthorizationChannel(policyHints.authorizationChannel);
  }
  if (requestedAction === 'story_edit' || objectType === 'story_object') {
    return AUTHORIZATION_CHANNELS.STORY_EDIT;
  }
  if (requestedAction === 'batch_read') {
    return AUTHORIZATION_CHANNELS.BATCH_READ;
  }
  return AUTHORIZATION_CHANNELS.SINGLE_READ;
}

export function assertRemoteChannelAllowed(channel) {
  const policy = channelPolicyFor(channel);
  if (!policy.remoteAllowed) {
    throw new Error(`${channel} is local-only and cannot be triggered by the remote gateway`);
  }
  return true;
}
