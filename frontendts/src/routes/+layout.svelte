<script lang="ts">
import { base } from "$app/paths"; // Importa la variable 'base' aquí
import "../app.css";
import { onMount } from "svelte";
import { goto } from "$app/navigation"; // Para redirecciones del lado del cliente
import { page } from "$app/state";
import Page from "$lib/domain/Page";
import { authStore } from "$lib/services/AuthService";
import { log } from "$lib/services/LoggerService";

let { children } = $props();

// Rutas que no requieren autenticación
const PUBLIC_ROUTES = [
	`${base}/`, // Tu página de inicio puede ser pública
	`${base}/login`,
	`${base}/auth/google/callback`, // El callback de Google es crucial que sea público
	// Agrega aquí cualquier otra ruta que deba ser accesible sin login
];

$effect(() => {
	const currentPath = page.url.pathname;
	const isPublicRoute = PUBLIC_ROUTES.some((route) => currentPath === route);

	if (!$authStore.isAuthenticated && !isPublicRoute) {
		log.info(`User not authenticated. Redirecting ${currentPath} to login.`);
		goto(`${base}/login`, { state: new Page(currentPath) });
	}
});

onMount(() => {
	const unsubscribe = authStore.subscribe(async ($auth) => {
		const currentPath = page.url.pathname;
		const isPublicRoute = PUBLIC_ROUTES.some((route) => currentPath === route);

		if (!$auth.isAuthenticated && !isPublicRoute) {
			log.info(`User not authenticated. Redirecting ${currentPath} to login.`);
			await goto(`${base}/login`, { state: new Page(currentPath) });
		}
		// Opcional: si el token ha expirado (verificado por una API call a tu backend)
		// Puedes implementar una lógica de refresco o re-login aquí.
	});

	return unsubscribe; // Limpia la suscripción cuando el componente se destruye
});
</script>

<nav class="rounded-2xl bg-white border-gray-200 dark:bg-gray-900 borde">
  <div class="max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-3">
    <a class="flex items-center space-x-3 rtl:space-x-reverse" href="{base}/">
      <img alt="Flowbite Logo" class="h-8" src="{base}/my_health_2.avif"/>
      <span class="self-center text-2xl font-semibold whitespace-nowrap dark:text-white">My Health Record</span>
    </a>
    <button aria-controls="navbar-default" aria-expanded="false"
            class="inline-flex items-center p-2 w-10 h-10 justify-center text-sm text-gray-500 rounded-lg md:hidden hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600"
            data-collapse-toggle="navbar-default" type="button">
      <span class="sr-only">Open main menu</span>
      <svg aria-hidden="true" class="w-5 h-5" fill="none" viewBox="0 0 17 14" xmlns="http://www.w3.org/2000/svg">
        <path d="M1 1h15M1 7h15M1 13h15" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"
              stroke-width="2"/>
      </svg>
    </button>
    <div class="hidden w-full md:block md:w-auto" id="navbar-default">
      <ul
        class="font-medium flex flex-col p-4 md:p-0 mt-4 border border-gray-100 rounded-lg bg-gray-50 md:flex-row md:space-x-8 rtl:space-x-reverse md:mt-0 md:border-0 md:bg-white dark:bg-gray-800 md:dark:bg-gray-900 dark:border-gray-700">
        <li>
          <a aria-current="page"
             class="block py-2 px-3 text-white bg-blue-700 rounded-sm md:bg-transparent md:text-blue-700 md:p-0 dark:text-white md:dark:text-blue-500"
             href="{base}/">Home</a>
        </li>
        <li>
          <a
            class="block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent"
            href="{base}/search">Busqueda</a>
        </li>
        <li>
          <a
            class="block py-2 px-3 text-gray-900 rounded-sm hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-blue-700 md:p-0 dark:text-white md:dark:hover:text-blue-500 dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent"
            href="#">Contactenos</a>
        </li>
      </ul>
    </div>
  </div>
</nav>

<main class="pt-6 mx-auto max-w-7xl sm:px-6 lg:px-8 ">
  <div class=" bg-white rounded-2xl shadow-lg p-6 md:p-8">
    {@render children()}
  </div>
</main>
