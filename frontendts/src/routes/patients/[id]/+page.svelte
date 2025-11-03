<script lang="ts">
import dayjs from "dayjs";
import { page } from "$app/state";
import ConfirmModal from "$lib/components/organisms/ConfirmModal.svelte";
import PatientConsultationList from "$lib/components/organisms/PatientConsultationList.svelte";
import PatientDetail from "$lib/components/organisms/PatientDetail.svelte";
import Appointment, { State } from "$lib/domain/Appointment";
import MedicalConditions from "$lib/domain/MedicalConditions";
import Patient from "$lib/domain/Patient";
import { log } from "$lib/services/LoggerService";

let patient: Patient =
  Object.keys(page.state).length === 0
    ? new Patient(
        "13",
        "Hector",
        "Malpica",
        0,
        "Gallegos",
        undefined,
        undefined,
        undefined,
        new MedicalConditions(true),
      )
    : (page.state as Patient);
log.debug("patient in page state:", patient);
let appointments = [
  dayjs("2025-01-01T23:35:01"),
  dayjs("2025-03-01T23:35:01"),
].map((d) => new Appointment(0, d, State.Booked, []));

let reset = $state(false);
let isEditing = $state(false);
let openModal = $state(false);

let full_name = patient.firstName;
log.log("patient in page, full_name: ", JSON.stringify(patient));
</script>

<div>
  <header class="mb-8">
    <h1 class="flex-none text-3xl tracking-tight text-gray-900">Ficha del Paciente: <span
      class="font-bold">{full_name}</span></h1>
    <div class="hidden sm:flex sm:flex-col sm:items-end">
      <!--        <img class="h-12 w-12 rounded-full bg-gray-50" src={&patient.clone().avatar.unwrap_or("".to_string())} alt="" />-->
    </div>
  </header>
  <main>
    <ConfirmModal
      bind:isOpen={openModal}
      callback={(confirm) => {if(!confirm) reset = true;}}/>
    <div class="mb-8">
      <PatientDetail
        bind:reset={reset}
        patient={patient}
        setEditing={ (editing) => isEditing = editing }/>
    </div>
    <div class="mb-8"
         on:click|capture={(e:Event) => {
             if (isEditing) {
               e.preventDefault()
               e.stopPropagation();
               openModal = true;
             }
           }}
         role="list">
      <PatientConsultationList appointments={appointments} patient={patient}/>
    </div>
  </main>
</div>
