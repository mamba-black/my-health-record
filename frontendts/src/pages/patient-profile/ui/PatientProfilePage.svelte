<script lang="ts">
import dayjs from "dayjs";
import { page } from "$app/state";
import { EncounterEntity, State } from "$entities/encounter";
import { MedicalConditions, Patient } from "$entities/patient";
import { log } from "$shared/lib/logger";
import { ConfirmModal } from "$shared/ui";
import { EncounterList } from "$widgets/encounter-list";
import { PatientDetail } from "$widgets/patient-detail";

const patient: Patient =
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

const encounterEntities = [
  dayjs("2025-01-01T23:35:01"),
  dayjs("2025-03-01T23:35:01"),
].map((d) => new EncounterEntity(0, d, State.Booked, []));

let reset = $state(false);
let isEditing = $state(false);
let openModal = $state(false);

const full_name = patient.firstName;
log.log("patient in page, full_name: ", JSON.stringify(patient));
</script>

<div>
  <header class="mb-8">
    <h1 class="flex-none text-3xl tracking-tight text-gray-900">
      Ficha del Paciente: <span class="font-bold">{full_name}</span>
    </h1>
    <div class="hidden sm:flex sm:flex-col sm:items-end"></div>
  </header>
  <main>
    <ConfirmModal
      bind:isOpen={openModal}
      callback={(confirm) => {
        if (!confirm) reset = true;
      }}
    />
    <div class="mb-8">
      <PatientDetail
        bind:reset={reset}
        patient={patient}
        setEditing={(editing) => (isEditing = editing)}
      />
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="mb-8"
      onclickcapture={(e: Event) => {
        if (isEditing) {
          e.preventDefault();
          e.stopPropagation();
          openModal = true;
        }
      }}
      onkeydowncapture={(e: KeyboardEvent) => {
        if (isEditing && e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          openModal = true;
        }
      }}
      role="region"
      aria-label="Historial de atenciones"
    >
      <EncounterList encounterEntities={encounterEntities} patient={patient}/>
    </div>
  </main>
</div>
