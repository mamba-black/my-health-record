<script lang="ts">
  import { page } from "$app/state";
  import PatientConsultationList from "$lib/components/organisms/PatientConsultationList.svelte";
  import PatientDetail from "$lib/components/organisms/PatientDetail.svelte";
  import Patient from "$lib/domain/Patient";
  import {log} from "$lib/services/LoggerService";
  import ConfirmModal from "$lib/components/organisms/ConfirmModal.svelte";
  import MedicalConditions from "$lib/domain/MedicalConditions";
  import dayjs from "dayjs";
  import Appointment, {State} from "$lib/domain/Appointment";

  let patient: Patient = Object.keys(page.state).length === 0
    ? new Patient("13", "Hector", "Malpica", 0, "Gallegos", undefined, undefined, undefined, new MedicalConditions(true))
    : page.state as Patient ;
  log.debug("patient in page state:", patient);
  let appointments = [dayjs("2025-01-01T23:35:01"), dayjs("2025-03-01T23:35:01")]
    .map(d => new Appointment(0, d, State.Booked, []));

  let reset = $state(false);
  let isEditing = $state(false);
  let openModal = $state(false);


  let full_name = patient.firstName;
  log.log("patient in page, full_name: ", JSON.stringify(patient));

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
    <ConfirmModal
      bind:isOpen={openModal}
      callback={(confirm) => {if(!confirm) reset = true;}} />
    <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
      <div class="px-10 py-10 bg-white rounded-2xl">
        <PatientDetail
          patient={patient}
          bind:reset={reset}
          setEditing={ (editing) => isEditing = editing }/>
      </div>
    </div>
    <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
      <div class="px-10 py-10 bg-white rounded-2xl"
           on:click|capture={(e:Event) => {
             if (isEditing) {
               openModal = true;
               e.stopPropagation();
             }
           }}>
        <PatientConsultationList patient={patient} appointments={appointments} />
      </div>
    </div>
  </main>
</div>
