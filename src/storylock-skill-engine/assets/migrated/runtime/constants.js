export const STORY_NODE_COUNT = 24;
export const BASIC_READ_THRESHOLD = 6;
export const BATCH_READ_THRESHOLD = 12;
export const MODIFY_THRESHOLD = 22;
export const BASIC_GRID_DISPLAY_SLOTS = 9;
export const BASIC_READ_OBJECT_LIMIT = 1;
export const CHALLENGE_TTL_MS = 5 * 60 * 1000;
export const SESSION_TTL_MS = 10 * 60 * 1000;
export const BASIC_SESSION_READS = 3;
export const BATCH_SESSION_READS = 32;
export const EXPORT_VERSION = "storylock-v1";
export const BASIC_MIN_CHALLENGE_BITS = 30;
export const BASIC_MIN_DIFFICULTY_BITS = 8;

export const DEFAULT_VERIFY_POLICY = Object.freeze({
  caseSensitive: false,
  ignoreLeadingTrailingSpaces: true,
  ignoreAllSpaces: false,
  distinguishFullWidthHalfWidth: false,
});

export const ROOT_KEY_KDF = Object.freeze({
  iterations: 3,
  parallelism: 1,
  memorySize: 4096,
  hashLength: 32,
});

export const QUESTION_KEY_KDF = Object.freeze({
  iterations: 1,
  parallelism: 1,
  memorySize: 256,
  hashLength: 32,
});

export const RECOVERY_KEY_KDF = Object.freeze({
  iterations: 3,
  parallelism: 1,
  memorySize: 4096,
  hashLength: 32,
});

export const PROD_ROOT_MIN_MEMORY_KIB = 64 * 1024;
export const PROD_ROOT_MIN_ITERATIONS = 3;
export const PROD_QUESTION_MIN_MEMORY_KIB = 16 * 1024;
export const PROD_QUESTION_MIN_ITERATIONS = 2;
