<script lang="ts">
import { clearAuth } from "$entities/session";
import { log } from "$shared/lib/logger";

let {
  label = "Cerrar sesión",
  className = "",
}: {
  label?: string;
  className?: string;
} = $props();

const handleLogout = () => {
  log.info("Cerrando sesión del usuario");
  clearAuth();
};
</script>

<button
  onclick={handleLogout}
  type="button"
  class={`inline-flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-red-700 bg-red-50 border border-red-200 rounded-lg hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-red-300 dark:bg-red-900/30 dark:text-red-300 dark:border-red-800 dark:hover:bg-red-900/50 cursor-pointer transition-colors ${className}`}
>
  <svg
    aria-hidden="true"
    class="w-4 h-4"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    viewBox="0 0 24 24"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
  <span>{label}</span>
</button>
