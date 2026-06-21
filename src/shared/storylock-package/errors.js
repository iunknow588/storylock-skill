export class StoryLockPackageError extends Error {
  constructor({ code, message, path = "$", level = "error", suggestion = "" }) {
    super(message);
    this.name = "StoryLockPackageError";
    this.code = code;
    this.path = path;
    this.level = level;
    this.suggestion = suggestion;
  }

  toIssue() {
    return {
      code: this.code,
      level: this.level,
      message: this.message,
      path: this.path,
      suggestion: this.suggestion,
    };
  }
}

export function createIssue({ code, message, path = "$", level = "error", suggestion = "" }) {
  return {
    code,
    level,
    message,
    path,
    suggestion,
  };
}

export function toIssue(error) {
  if (error instanceof StoryLockPackageError) {
    return error.toIssue();
  }
  return createIssue({
    code: "SLP-999",
    message: error?.message || "Unknown StoryLock package error",
    suggestion: "Check the package input and retry.",
  });
}
