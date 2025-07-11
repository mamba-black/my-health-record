<script lang="ts">
  import SearchList from "$lib/components/organisms/SearchList.svelte";
  import SearchInput from "$lib/components/organisms/SearchInput.svelte";
  import type Patient from "$lib/domain/Patient";

  let {patients = $bindable()}: { patients: Patient[] } = $props();
</script>

<div>
  <h1 class="text-3xl font-bold tracking-tight text-gray-900 pb-7 ">
    Busqueda de pacientes
  </h1>
  <SearchInput bind:patients={patients}/>
  <SearchList bind:patients={patients}/>
</div>
