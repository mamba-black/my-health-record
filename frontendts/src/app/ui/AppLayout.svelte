<script lang="ts">
import "../styles/app.css";
import { onMount, type Snippet } from "svelte";
import { goto } from "$app/navigation";
import { page } from "$app/state";
import {
  authStore,
  canRegisterPatients,
  canSearchPatients,
  profileContext,
} from "$entities/session";
import { RequirementsDialog } from "$features/profile";
import {
  ACCESS_DENIED,
  CALLBACK,
  CLINIC_PUBLIC_PREFIX,
  HISTORY,
  LOGIN,
  Page,
  REGISTER_PATIENT,
  ROOT,
  SEARCH,
  SIGNUP,
} from "$shared/config";
import { log } from "$shared/lib";
import { Navbar } from "$widgets/navbar";

let { children }: { children: Snippet } = $props();

const PUBLIC_ROUTES = [ROOT, LOGIN, SIGNUP, CALLBACK];
// La página pública de una clínica (`/c/<subdominio>`) se ve sin sesión.
const isPublicPath = (path: string) =>
  PUBLIC_ROUTES.some((route) => path === route) || path.startsWith(CLINIC_PUBLIC_PREFIX);
const isPublicRoute = $derived(isPublicPath(page.url.pathname));

// Rutas por rol. Es solo presentación: el servidor aplica la autorización de todos modos.
// Registrar pacientes: dueño, administrador o recepcionista activos.
const needsReception = $derived(page.url.pathname.startsWith(REGISTER_PATIENT));
// Buscar y ver pacientes: médico o recepcionista activos en una clínica activa.
const PATIENT_ROUTES = [SEARCH, HISTORY];
const needsPractitioner = $derived(
  !needsReception && PATIENT_ROUTES.some((route) => page.url.pathname.startsWith(route)),
);
const roleResolved = $derived(
  (!needsPractitioner && !needsReception) || $profileContext !== null,
);
const allowedByRole = $derived(
  needsReception ? $canRegisterPatients : !needsPractitioner || $canSearchPatients,
);

$effect(() => {
  if ($authStore.isAuthenticated && roleResolved && !allowedByRole) {
    log.info(`Rol sin acceso a ${page.url.pathname}. Redirigiendo a ${ACCESS_DENIED}.`);
    goto(ACCESS_DENIED);
  }
});

$effect(() => {
  const currentPath = page.url.pathname;

  if (!$authStore.isAuthenticated && !isPublicRoute) {
    log.info(`User not authenticated. Redirecting ${currentPath} to login.`);
    goto(LOGIN, { state: new Page(currentPath) });
  }
});

onMount(() => {
  const unsubscribe = authStore.subscribe(async ($auth) => {
    const currentPath = page.url.pathname;
    const publicRoute = isPublicPath(currentPath);

    if (!$auth.isAuthenticated && !publicRoute) {
      log.info(`User not authenticated. Redirecting ${currentPath} to login.`);
      await goto(LOGIN, { state: new Page(currentPath) });
    }
  });

  return unsubscribe;
});
</script>

<Navbar/>
<RequirementsDialog />

<main class="pt-6 mx-auto max-w-7xl sm:px-6 lg:px-8">
  <div class="bg-white rounded-2xl shadow-lg p-6 md:p-8">
    {#if isPublicRoute}
      {@render children()}
    {:else if $authStore.isAuthenticated}
      {#if !roleResolved}
        <p class="text-sm text-gray-500">Cargando...</p>
      {:else if allowedByRole}
        {@render children()}
      {/if}
    {/if}
  </div>
</main>
