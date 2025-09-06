import { default as logInstance } from "loglevel";
import { dev } from "$app/environment";

if (dev) {
  logInstance.setLevel("debug");
} else {
  logInstance.setLevel("warn");
}

// Personaliza cómo loglevel mapea sus métodos a los métodos de consola nativos
logInstance.methodFactory = (methodName, logLevel, loggerName) => {
  // Obtiene el método de consola nativo correspondiente.
  // Por defecto, loglevel mapea 'debug', 'info', 'warn', 'error' a console.log, console.warn, console.error.
  // Aquí estamos sobrescribiendo eso para 'debug'.
  const rawMethod = (console as any)[methodName];

  // Si el método de consola nativo existe y no es 'log' (para evitar que debug, info caigan en log por defecto)
  // o si el methodName es 'debug' y console.debug existe
  if (methodName === "debug" && typeof console.debug === "function") {
    return console.debug.bind(console); // Mapea logInstance.debug() a console.debug()
  } else if (typeof rawMethod === "function") {
    // Para otros métodos como 'info', 'warn', 'error', usa sus métodos nativos correspondientes.
    // loglevel ya hace esto para warn y error, pero para info por defecto usa log.
    // Aquí puedes decidir si quieres que info use console.info si existe, o seguir usando console.log
    if (methodName === "info" && typeof console.info === "function") {
      return console.info.bind(console);
    }
    return rawMethod.bind(console);
  } else {
    // Si no hay un método nativo correspondiente, usa console.log como fallback
    return console.log.bind(console);
  }
};

// Vuelve a aplicar los métodos después de cambiar methodFactory
// Esto es importante para que los cambios en methodFactory tengan efecto en las llamadas de log existentes.
logInstance.enableAll(); // O loggerInstance.setLevel(loggerInstance.getLevel());

export const log = logInstance;
