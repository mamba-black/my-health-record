<!--
@component
- Es una prueba
-->

<script lang="ts">
  import Patient from "./Patient";
  import SubmitButton from "./SubmitButton.svelte";

  let { patients = $bindable() }: { patients: Patient[] } = $props();
  let name: string = $state("");

  const findPatient = (event: Event) => {
    event.preventDefault();
    name = name.toUpperCase();
    patients = [new Patient("123321", "Picard", 85, "miuler@gmail.com"), new Patient("456465", "Jan", 85, "miuler@gmail.com")];
  };
</script>

<form onsubmit={findPatient}>
  <label
    for="default-search"
    class="mb-2 text-sm font-medium text-gray-900 sr-only dark:text-white"
  >Search</label>
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
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        >
        </path>
      </svg>
    </div>
    <input
      type="search"
      id="default-search"
      class="w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
      placeholder="Busqueda de pacientes, nombre..."
      required={true}
      bind:value={name}
    />
    <span class="absolute right-2.5 pt-2">
      <SubmitButton label={"Busqueda"} />
    </span>
  </div>
</form>
