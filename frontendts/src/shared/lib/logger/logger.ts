import { default as logInstance } from "loglevel";
import { dev } from "$app/environment";

if (dev) {
  logInstance.setLevel("debug");
} else {
  logInstance.setLevel("warn");
}

// Personaliza cómo loglevel mapea sus métodos a los métodos de consola nativos
logInstance.methodFactory = (methodName, _logLevel, _loggerName) => {
  const rawMethod = (console as unknown as Record<string, unknown>)[methodName];

  if (methodName === "debug" && typeof console.debug === "function") {
    return console.debug.bind(console);
  } else if (typeof rawMethod === "function") {
    if (methodName === "info" && typeof console.info === "function") {
      return console.info.bind(console);
    }
    return (rawMethod as (...args: unknown[]) => void).bind(console);
  } else {
    return console.log.bind(console);
  }
};

logInstance.enableAll();

export const log = logInstance;
