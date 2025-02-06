<script lang="ts">
  import Grid from "$lib/components/organisms/Grid.svelte";
  import SearchInput from "$lib/components/organisms/SearchInput.svelte";
  import type Patient from "$lib/domain/Patient";

  let { patients = $bindable() }: { patients: Patient[] } = $props();
</script>

<header class="bg-white shadow">
  <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
    <h1 class="text-3xl font-bold tracking-tight text-gray-900">
      Busqueda de pacientes
    </h1>
  </div>
</header>

<main>
  <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
    <div class="px-10 py-10 bg-white rounded-2xl">
      <SearchInput bind:patients={patients} />
      <Grid bind:patients={patients} />
    </div>
  </div>
</main>
