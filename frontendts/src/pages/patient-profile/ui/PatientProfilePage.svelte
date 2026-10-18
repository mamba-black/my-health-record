<script lang="ts">
import dayjs from "dayjs";
import { page } from "$app/state";
import { EncounterEntity, State } from "$entities/encounter";
import { getPatientById, type Patient } from "$entities/patient";
import { log } from "$shared/lib/logger";
import { ConfirmModal } from "$shared/ui";
import { EncounterList } from "$widgets/encounter-list";
import { PatientDetail } from "$widgets/patient-detail";

const initialPatient: Patient | null =
  page.state && (page.state as Patient).id === page.params.id
    ? (page.state as Patient)
    : null;

let patient = $state<Patient | null>(initialPatient);
let isLoading = $state<boolean>(!initialPatient);
let errorMessage = $state<string | null>(null);

let reset = $state(false);
let isEditing = $state(false);
let openModal = $state(false);

const encounterEntities = [
  dayjs("2025-01-01T23:35:01"),
  dayjs("2025-03-01T23:35:01"),
].map((d) => new EncounterEntity(0, d, State.Booked, []));

$effect(() => {
  const patientId = page.params.id;
  if (!patientId) return;

  if (patient && patient.id === patientId) return;

  isLoading = true;
  errorMessage = null;

  getPatientById(patientId)
    .then((loadedPatient) => {
      log.debug("Patient loaded successfully:", loadedPatient);
      patient = loadedPatient;
    })
    .catch((err) => {
      log.error("Failed to load patient by id:", err);
      errorMessage = "No se pudo cargar la información del paciente.";
    })
    .finally(() => {
      isLoading = false;
    });
});

const fullName = $derived(
  patient
    ? `${patient.firstName} ${patient.lastName} ${patient.secondLastName ?? ""}`.trim()
    : "",
);
</script>

<div>
  {#if isLoading}
    <div class="flex items-center justify-center p-12">
      <p class="text-gray-500 text-lg">Cargando datos del paciente...</p>
    </div>
  {:else if errorMessage}
    <div class="p-6 bg-red-50 border border-red-200 text-red-700 rounded-lg">
      <h2 class="text-lg font-semibold">Error</h2>
      <p class="mt-1">{errorMessage}</p>
    </div>
  {:else if patient}
    <header class="mb-8">
      <h1 class="flex-none text-3xl tracking-tight text-gray-900">
        Ficha del Paciente: <span class="font-bold">{fullName}</span>
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
  {/if}
</div>
