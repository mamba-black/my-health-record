<script lang="ts" xmlns="http://www.w3.org/1999/html">
  import MedicalConditions from "$lib/components/organisms/MedicalConditions.svelte";
  import {log} from "$lib/services/LoggerService";
  import type Patient from "$lib/domain/Patient";
  import type Appointment from "$lib/domain/Appointment";
  import STitle from "$lib/components/atoms/STitle.svelte";
  // import {Edra} from "$lib/components/edra/headless";

  let {isOpen = $bindable(), patient, appointment}: {
    isOpen: boolean,
    patient: Patient,
    appointment: Appointment
  } = $props();

  function onClick(e: Event) {
    e.preventDefault();
    isOpen = !isOpen;
    log.debug("isOpen", isOpen);
  }
</script>

{#if isOpen}
  <div class="max-w-4xl mx-auto bg-white shadow-lg rounded-lg p-6 md:p-8">
<!--  <div class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-4/5 max-w-[800px] bg-white p-6 rounded-lg shadow-lg border-gray-400 p-1"> -->

    <h1 id="current-appointment-title" class="text-3xl font-bold text-gray-800 text-center mb-6">Detalle de la Atención
      ({appointment.date})</h1>

    <MedicalConditions medical_conditions={patient.medicalConditions} readonly=true></MedicalConditions>

    <div class="mb-8">
      <STitle>Sintomas</STitle>
      <textarea autofocus
                class="[field-sizing:content] w-full bg-yellow-50 border border-yellow-200 text-yellow-800 p-4 rounded-md">Dolor de cabeza
      </textarea>
    </div>

    <div class="mb-8">
      <STitle>Diagnostico</STitle>
      <textarea
        class="[field-sizing:content] w-full bg-purple-50 border border-purple-200 text-purple-800 p-4 rounded-md">Paciente presenta cuadro de resfriado común, con congestión nasal severa, tos productiva y dolor de garganta. Se auscultan sibilancias leves. Se recomienda reposo, hidratación abundante y paracetamol para la fiebre.
      </textarea>
    </div>

    <div class="mb-8">
      <STitle>Tratamiento</STitle>
      <textarea class="[field-sizing:content] w-full bg-emerald-50 border border-emerald-200 text-emerald-800 p-4 rounded-md">- Paracetamos
        - Ibuprofeno
      </textarea>
    </div>

    <div class="justify-self-end">
      <button onclick={onClick} class="button-submit">Cerrar</button>
    </div>
  </div>
{/if}
