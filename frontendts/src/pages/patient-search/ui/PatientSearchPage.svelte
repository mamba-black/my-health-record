<script lang="ts">
import { type Patient, SearchList } from "$entities/patient";
import { SearchInput } from "$features/patient-search";

let patients: Patient[] = $state([]);
</script>

<div>
  <h1 class="text-3xl font-bold tracking-tight text-gray-900 pb-7">
    Busqueda de pacientes
  </h1>
  <SearchInput bind:patients={patients}/>
  <SearchList bind:patients={patients}/>
</div>
