<script lang="ts">
import { goto } from "$app/navigation";
import type { EncounterEntity } from "$entities/encounter";
import { MedicalConditionsCard, type Patient } from "$entities/patient";
import { HISTORY } from "$shared/config/routes";
import { log } from "$shared/lib/logger";
import { STitle } from "$shared/ui";

let {
  isOpen = $bindable(),
  patient,
  encounterEntity = $bindable(),
}: {
  isOpen: boolean;
  patient: Patient;
  encounterEntity: EncounterEntity;
} = $props();

function onClick(e: Event) {
  e.preventDefault();
  isOpen = !isOpen;
  log.debug("isOpen", isOpen);
  // GRABAR EN EL BACKEND
  goto(`${HISTORY}/${patient.id}/`, { state: patient });
}
</script>

{#if isOpen}
  <div>
    <h1 id="current-encounter-title" class="text-3xl font-bold text-gray-800 text-center mb-6">
      Detalle de la Atención ({encounterEntity.date})
    </h1>

    <MedicalConditionsCard medical_conditions={patient.medicalConditions} readonly={true}/>

    <div class="mb-8">
      <STitle>Sintomas</STitle>
      <textarea
        class="[field-sizing:content] w-full bg-yellow-50 border border-yellow-200 text-yellow-800 p-4 rounded-md"
      >Dolor de cabeza</textarea>
    </div>

    <div class="mb-8">
      <STitle>Diagnostico</STitle>
      <textarea
        class="[field-sizing:content] w-full bg-purple-50 border border-purple-200 text-purple-800 p-4 rounded-md"
      >Paciente presenta cuadro de resfriado común, con congestión nasal severa, tos productiva y dolor de garganta. Se auscultan sibilancias leves. Se recomienda reposo, hidratación abundante y paracetamol para la fiebre.</textarea>
    </div>

    <div class="mb-8">
      <STitle>Tratamiento</STitle>
      <textarea
        class="[field-sizing:content] w-full bg-emerald-50 border border-emerald-200 text-emerald-800 p-4 rounded-md"
      >- Paracetamos
        - Ibuprofeno</textarea>
    </div>

    <div class="justify-self-end">
      <button type="button" onclick={onClick} class="button-submit">Cerrar</button>
    </div>
  </div>
{/if}
