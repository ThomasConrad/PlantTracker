/* eslint-disable no-console */
/**
 * Structured browser console logger for API debugging.
 *
 * Outputs contextual information (endpoint, status, timing, payloads)
 * in grouped console calls. Controlled by log level:
 *
 *   localStorage.setItem("planty:logLevel", "debug") — verbose
 *   localStorage.setItem("planty:logLevel", "info")  — default in dev
 *   localStorage.setItem("planty:logLevel", "warn")  — default in prod
 *   localStorage.setItem("planty:logLevel", "error") — errors only
 *   localStorage.setItem("planty:logLevel", "off")   — silent
 */

type LogLevel = "debug" | "info" | "warn" | "error" | "off";

const LEVEL_ORDER: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
  off: 4,
};

const STORAGE_KEY = "planty:logLevel";

function getLevel(): LogLevel {
  try {
    const stored = localStorage.getItem(STORAGE_KEY) as LogLevel | null;
    if (stored && stored in LEVEL_ORDER) return stored;
  } catch {
    // localStorage unavailable (private browsing, SSR, etc.)
  }
  return import.meta.env.DEV ? "info" : "warn";
}

function shouldLog(level: LogLevel): boolean {
  return LEVEL_ORDER[level] >= LEVEL_ORDER[getLevel()];
}

// ─── Public API ────────────────────────────────────────────────────────────

export interface ApiLogContext {
  method: string;
  url: string;
  status?: number;
  durationMs?: number;
  requestBody?: unknown;
  responseBody?: unknown;
  error?: unknown;
  extra?: Record<string, unknown>;
}

/** Log a successful API response */
export function logApiSuccess(ctx: ApiLogContext): void {
  if (!shouldLog("debug")) return;

  const tag = `%c[API] %c${ctx.method} ${ctx.url} %c${ctx.status} %c${ctx.durationMs}ms`;
  const styles = [
    "color: #6b7280; font-weight: bold",
    "color: #2563eb",
    "color: #16a34a; font-weight: bold",
    "color: #6b7280",
  ];

  if (shouldLog("debug") && (ctx.requestBody || ctx.responseBody)) {
    console.groupCollapsed(tag, ...styles);
    if (ctx.requestBody) console.log("Request:", ctx.requestBody);
    if (ctx.responseBody) console.log("Response:", ctx.responseBody);
    console.groupEnd();
  } else {
    console.debug(tag, ...styles);
  }
}

/** Log an API error with full context */
export function logApiError(ctx: ApiLogContext): void {
  if (!shouldLog("error")) return;

  const statusLabel = ctx.status ? `${ctx.status}` : "NETWORK";
  const tag = `[API ERROR] ${ctx.method} ${ctx.url} → ${statusLabel}`;

  console.groupCollapsed(`%c${tag}`, "color: #dc2626; font-weight: bold");
  if (ctx.durationMs != null) console.log("Duration:", `${ctx.durationMs}ms`);
  if (ctx.requestBody) console.log("Request body:", ctx.requestBody);
  if (ctx.responseBody) console.log("Response body:", ctx.responseBody);
  if (ctx.error) console.error("Error:", ctx.error);
  if (ctx.extra) console.log("Extra:", ctx.extra);
  console.trace("Stack");
  console.groupEnd();
}

/** Log a warning (e.g., slow response, retry, degraded state) */
export function logWarn(message: string, data?: Record<string, unknown>): void {
  if (!shouldLog("warn")) return;
  if (data) {
    console.warn(`[Planty] ${message}`, data);
  } else {
    console.warn(`[Planty] ${message}`);
  }
}

/** Log informational events (e.g., auth state changes, offline/online) */
export function logInfo(message: string, data?: Record<string, unknown>): void {
  if (!shouldLog("info")) return;
  if (data) {
    console.info(`%c[Planty]%c ${message}`, "color: #6b7280; font-weight: bold", "color: inherit", data);
  } else {
    console.info(`%c[Planty]%c ${message}`, "color: #6b7280; font-weight: bold", "color: inherit");
  }
}

/** Log verbose debug info */
export function logDebug(message: string, data?: Record<string, unknown>): void {
  if (!shouldLog("debug")) return;
  if (data) {
    console.debug(`[Planty] ${message}`, data);
  } else {
    console.debug(`[Planty] ${message}`);
  }
}
