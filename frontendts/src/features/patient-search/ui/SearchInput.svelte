<script lang="ts">
import type { Patient } from "$entities/patient";
import { log } from "$shared/lib/logger";
import { SubmitButton } from "$shared/ui";
import { searchPatients } from "../api/search-patient.api";

let { patients = $bindable() }: { patients: Patient[] } = $props();
let name: string = $state("");
let isLoading: boolean = $state(false);
let errorMessage: string | null = $state(null);

const findPatient = async (event: Event) => {
  event.preventDefault();
  isLoading = true;
  errorMessage = null;

  try {
    patients = await searchPatients(name);
  } catch (error) {
    log.error("Error al buscar pacientes:", error);
    errorMessage =
      "Ocurrió un error al buscar pacientes. Por favor intente nuevamente.";
  } finally {
    isLoading = false;
  }
};
</script>

<form onsubmit={findPatient} class="space-y-2">
  <label
    class="mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white"
    for="default-search"
  >Buscar pacientes</label>
  <div class="relative items-center h-14">
    <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
      <svg
        aria-hidden="true"
        class="w-5 h-5 text-gray-500 dark:text-gray-400"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
        >
        </path>
      </svg>
    </div>
    <input
      bind:value={name}
      class="w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
      id="default-search"
      name="q"
      placeholder="Busqueda de pacientes, nombre..."
      required={true}
      type="search"
      disabled={isLoading}
    />
    <span class="absolute right-2.5 pt-2">
      <SubmitButton label={isLoading ? "Buscando..." : "Busqueda"}/>
    </span>
  </div>

  {#if errorMessage}
    <div
      role="alert"
      aria-live="polite"
      class="p-3 text-sm text-red-700 bg-red-100 rounded-lg dark:bg-red-900/30 dark:text-red-400"
    >
      {errorMessage}
    </div>
  {/if}
</form>
