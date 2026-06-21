import { createIssue } from "./errors.js";

const OBJECT_ID_PATTERN = /^[a-z0-9][a-z0-9_-]*(?:\/[a-z0-9][a-z0-9_-]*){3}$/u;

export function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

export function validateRequiredString(value, path, issues, code = "SLP-101") {
  if (typeof value !== "string" || value.trim() === "") {
    issues.push(createIssue({
      code,
      path,
      message: `${path} must be a non-empty string.`,
      suggestion: "Provide a non-empty string value.",
    }));
    return false;
  }
  return true;
}

export function validateOptionalString(value, path, issues) {
  if (value == null) {
    return true;
  }
  return validateRequiredString(value, path, issues);
}

export function validateArray(value, path, issues, { minItems = 0 } = {}) {
  if (!Array.isArray(value)) {
    issues.push(createIssue({
      code: "SLP-102",
      path,
      message: `${path} must be an array.`,
      suggestion: "Use an array value.",
    }));
    return false;
  }
  if (value.length < minItems) {
    issues.push(createIssue({
      code: "SLP-103",
      path,
      message: `${path} must contain at least ${minItems} item(s).`,
      suggestion: "Add the required items before exporting.",
    }));
    return false;
  }
  return true;
}

export function validateObject(value, path, issues) {
  if (!isPlainObject(value)) {
    issues.push(createIssue({
      code: "SLP-104",
      path,
      message: `${path} must be an object.`,
      suggestion: "Use an object value.",
    }));
    return false;
  }
  return true;
}

export function validateObjectId(value, path, issues) {
  if (!validateRequiredString(value, path, issues, "SLP-201")) {
    return false;
  }
  if (!OBJECT_ID_PATTERN.test(value)) {
    issues.push(createIssue({
      code: "SLP-202",
      path,
      message: `${path} must be a four-segment objectId.`,
      suggestion: "Use <resourceKind>/<providerId>/<instanceSegment>/<role>.",
    }));
    return false;
  }
  return true;
}

export function uniqueBy(items, keyFn, path, label, issues) {
  const seen = new Map();
  for (let index = 0; index < items.length; index += 1) {
    const key = keyFn(items[index], index);
    if (!key) {
      continue;
    }
    if (seen.has(key)) {
      issues.push(createIssue({
        code: "SLP-203",
        path: `${path}[${index}]`,
        message: `Duplicate ${label}: ${key}.`,
        suggestion: `Keep ${label} unique inside ${path}.`,
      }));
    } else {
      seen.set(key, index);
    }
  }
}
