export class StoryLockError extends Error {
  constructor(message, code = "STORY_LOCK_ERROR") {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
  }
}

export class ValidationError extends StoryLockError {
  constructor(message) {
    super(message, "VALIDATION_ERROR");
  }
}

export class AuthorizationError extends StoryLockError {
  constructor(message) {
    super(message, "AUTHORIZATION_ERROR");
  }
}

export class ChallengeError extends StoryLockError {
  constructor(message) {
    super(message, "CHALLENGE_ERROR");
  }
}

export class StorageError extends StoryLockError {
  constructor(message) {
    super(message, "STORAGE_ERROR");
  }
}

export class IntegrityError extends StoryLockError {
  constructor(message) {
    super(message, "INTEGRITY_ERROR");
  }
}

export class CloudBackupIntegrityError extends IntegrityError {
  constructor(message) {
    super(message);
    this.code = "CLOUD_BACKUP_INTEGRITY_ERROR";
  }
}
