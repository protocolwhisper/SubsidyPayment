import pino from 'pino';

export function createLogger(logLevel: string = process.env.LOG_LEVEL || 'info') {
  return pino({
    level: logLevel,
    timestamp: pino.stdTimeFunctions.isoTime,
    base: undefined,
  });
}

export const logger = createLogger();
