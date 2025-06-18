<script lang="ts">
  import MedicalConditions from "$lib/components/organisms/MedicalConditions.svelte";
  import {log} from "$lib/services/LoggerService";
  // import {Edra} from "$lib/components/edra/headless";

  let { isOpen = $bindable(), patient } = $props();
  // let { patient }: { patient: Patient } = $props();

  function onClick(e: Event) {
    e.preventDefault();
    isOpen = !isOpen;
    log.debug("isOpen", isOpen);
  }
</script>

{#if isOpen}
  <div
    class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-4/5 max-w-[800px] bg-white p-6 rounded-lg shadow-xl border border-gray-400 p-1"
  >
    <MedicalConditions medical_conditions={patient.medicalConditions} readonly=true> </MedicalConditions>
    <h3 class="col-span-3 text-2xl font-bold tracking-tight text-gray-900 pt-1 mt-6">Sintomas</h3>
    <textarea class="[field-sizing:content] w-full mt-5 border border-gray-400 p-1 rounded-md">Dolor de cabeza
    </textarea>
    <h3 class="col-span-3 text-2xl font-bold tracking-tight text-gray-900 pt-1 ">Diagnostico</h3>
    <textarea class="[field-sizing:content] w-full mt-5 border border-gray-400 p-1 rounded-md">Paciente presenta cuadro de resfriado común, con congestión nasal severa, tos productiva y dolor de garganta. Se auscultan sibilancias leves. Se recomienda reposo, hidratación abundante y paracetamol para la fiebre.
    </textarea>
    <h3 class="col-span-3 text-2xl font-bold tracking-tight text-gray-900 pt-1 mt-6 rounded-md">Tratamiento</h3>
<!--    <Edra>-->
<!--      - Paracetamos-->
<!--      - Ibuprofeno-->
<!--    </Edra>-->
    <textarea class="[field-sizing:content] w-full mt-5 border border-gray-400 p-1">- Paracetamos
- Ibuprofeno
    </textarea>
    <div class="justify-self-end">
      <button onclick={onClick} class="button-submit">Cerrar</button>
    </div>
  </div>
{/if}
