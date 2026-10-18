<script lang="ts">
import "../styles/app.css";
import { onMount, type Snippet } from "svelte";
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { authStore } from "$entities/session";
import { CALLBACK, LOGIN, Page, ROOT } from "$shared/config/routes";
import { log } from "$shared/lib/logger";
import { Navbar } from "$widgets/navbar";

let { children }: { children: Snippet } = $props();

const PUBLIC_ROUTES = [ROOT, LOGIN, CALLBACK];

$effect(() => {
  const currentPath = page.url.pathname;
  const isPublicRoute = PUBLIC_ROUTES.some((route) => currentPath === route);

  if (!$authStore.isAuthenticated && !isPublicRoute) {
    log.info(`User not authenticated. Redirecting ${currentPath} to login.`);
    goto(LOGIN, { state: new Page(currentPath) });
  }
});

onMount(() => {
  const unsubscribe = authStore.subscribe(async ($auth) => {
    const currentPath = page.url.pathname;
    const isPublicRoute = PUBLIC_ROUTES.some((route) => currentPath === route);

    if (!$auth.isAuthenticated && !isPublicRoute) {
      log.info(`User not authenticated. Redirecting ${currentPath} to login.`);
      await goto(LOGIN, { state: new Page(currentPath) });
    }
  });

  return unsubscribe;
});
</script>

<Navbar/>

<main class="pt-6 mx-auto max-w-7xl sm:px-6 lg:px-8">
  <div class="bg-white rounded-2xl shadow-lg p-6 md:p-8">
    {@render children()}
  </div>
</main>
