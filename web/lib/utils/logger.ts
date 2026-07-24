/**
 * Centralized logging utility for the Axon application.
 * Only outputs logs to the console if the `DEBUG` environment variable is set to 'true'.
 * This prevents sensitive information from leaking and keeps production logs clean.
 */
export const logger = {
  log: (...args: any[]) => {
    if (process.env.DEBUG === 'true') {
      console.log(...args);
    }
  },
  error: (...args: any[]) => {
    if (process.env.DEBUG === 'true') {
      console.error(...args);
    }
  },
  warn: (...args: any[]) => {
    if (process.env.DEBUG === 'true') {
      console.warn(...args);
    }
  },
  info: (...args: any[]) => {
    if (process.env.DEBUG === 'true') {
      console.info(...args);
    }
  }
};
