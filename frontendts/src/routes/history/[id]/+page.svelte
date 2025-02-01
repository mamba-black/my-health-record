<script lang="ts">
  import { page } from "$app/state";
  import PatientConsultationList from "$lib/components/organisms/PatientConsultationList.svelte";
  import PatientDetail from "$lib/components/organisms/PatientDetail.svelte";
  import type Patient from "$lib/Patient";

  let patient = page.state as Patient;
  let full_name = patient.firstName;
  console.log("patient in page, full_name: ", JSON.stringify(patient));
</script>

<div>
  <header class="bg-white shadow">
    <div class="flex justify-between mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
      <div class="flex gap-x-4">
        <h1 class="flex-none text-3xl font-bold tracking-tight text-gray-900 pt-1">Ficha del Paciente: {full_name}</h1>
      </div>
      <div class="hidden sm:flex sm:flex-col sm:items-end">
        <!--        <img class="h-12 w-12 rounded-full bg-gray-50" src={&patient.clone().avatar.unwrap_or("".to_string())} alt="" />-->
      </div>
    </div>
  </header>
  <main>
    <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
      <div class="px-10 py-10 bg-white rounded-2xl">
        <PatientDetail patient={patient} />
      </div>
    </div>
    <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
      <div class="px-10 py-10 bg-white rounded-2xl">
        <PatientConsultationList patient={patient} />
      </div>
    </div>
  </main>
</div>
