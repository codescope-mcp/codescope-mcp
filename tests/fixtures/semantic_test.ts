// Test file for semantic context detection

// Mock chronia library
const chronia = {
  now: () => new Date(),
  format: (date: Date) => date.toISOString(),
};

// Function using Date.now()
function getTimestamp(): number {
  return Date.now();
}

// Function using chronia.now()
function getCurrentDate(): Date {
  return chronia.now();
}

// Multiple usages in one function
function logWithTimestamp(message: string): void {
  const timestamp = Date.now();
  const formatted = chronia.format(new Date(timestamp));
  console.log(`[${formatted}] ${message}`);
}

// Property access (not method call)
const config = {
  timeout: 5000,
  retryCount: 3,
};

function getConfig() {
  const timeout = config.timeout;
  return timeout;
}

// Class with method calls
class TimerService {
  private startTime: number;

  constructor() {
    this.startTime = Date.now();
  }

  elapsed(): number {
    return Date.now() - this.startTime;
  }

  getCurrentTime(): Date {
    return chronia.now();
  }
}

// Arrow function with Date.now()
const recordEvent = (eventName: string) => {
  const time = Date.now();
  return { eventName, time };
};

// Export
export { getTimestamp, getCurrentDate, logWithTimestamp, getConfig, TimerService, recordEvent };
