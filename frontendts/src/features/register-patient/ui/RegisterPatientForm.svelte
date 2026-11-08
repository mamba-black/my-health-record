<script lang="ts">
import { type Patient, UserApiError } from "$shared/api";
import { log } from "$shared/lib";
import { registerPatient } from "../api/register-patient.api";

/**
 * Recepción registra a un paciente que no tiene cuenta (por ejemplo, porque llamó por
 * teléfono). Su expediente queda en la clínica y podrá vincularse con su cuenta después.
 */
let { onRegistered }: { onRegistered?: (patient: Patient) => void } = $props();

type DocumentType = "dni" | "ce" | "passport";

let givenName = $state("");
let familyName = $state("");
let secondFamilyName = $state("");
let documentType = $state<DocumentType>("dni");
let documentNumber = $state("");
let phone = $state("");
let birthDate = $state("");
let busy = $state(false);
let errorMessage = $state<string | null>(null);

const inputClass =
  "w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none";
const labelClass = "block text-sm font-medium text-gray-700 mb-1";

function identifierOf(type: DocumentType, value: string) {
  switch (type) {
    case "dni":
      return { identifierType: { oneofKind: "dni" as const, dni: value } };
    case "ce":
      return { identifierType: { oneofKind: "ce" as const, ce: value } };
    case "passport":
      return { identifierType: { oneofKind: "passport" as const, passport: value } };
  }
}

async function submit(event: Event) {
  event.preventDefault();
  errorMessage = null;
  const number = documentNumber.trim();
  if (!givenName.trim() || !familyName.trim() || !number || !phone.trim()) {
    errorMessage = "Completa nombres, apellido paterno, documento y teléfono.";
    return;
  }
  if (documentType === "dni" && !/^\d{8}$/.test(number)) {
    errorMessage = "El DNI debe tener 8 dígitos.";
    return;
  }
  busy = true;
  try {
    const patient = await registerPatient({
      givenName: givenName.trim(),
      familyName: familyName.trim(),
      secondFamilyName: secondFamilyName.trim() || undefined,
      identifier: identifierOf(documentType, number),
      phone: phone.trim(),
      birthDate: birthDate || undefined,
    });
    givenName = familyName = secondFamilyName = documentNumber = phone = birthDate = "";
    onRegistered?.(patient);
  } catch (error) {
    log.error("No se pudo registrar al paciente", error);
    if (error instanceof UserApiError && error.code === "ALREADY_EXISTS") {
      errorMessage = "La clínica ya tiene un paciente con ese documento: búscalo en «Búsqueda».";
    } else if (error instanceof UserApiError && error.code === "PERMISSION_DENIED") {
      errorMessage = "Solo la recepción, un administrador o el dueño de la clínica registran pacientes.";
    } else {
      errorMessage = error instanceof Error ? error.message : "No se pudo registrar al paciente.";
    }
  } finally {
    busy = false;
  }
}
</script>

<form class="space-y-4" data-testid="register-patient" onsubmit={submit}>
  {#if errorMessage}
    <div class="p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700" role="alert">
      {errorMessage}
    </div>
  {/if}

  <div>
    <label class={labelClass} for="patient-given-name">Nombres *</label>
    <input bind:value={givenName} class={inputClass} id="patient-given-name" required type="text" />
  </div>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
    <div>
      <label class={labelClass} for="patient-family-name">Apellido paterno *</label>
      <input bind:value={familyName} class={inputClass} id="patient-family-name" required type="text" />
    </div>
    <div>
      <label class={labelClass} for="patient-second-family-name">Apellido materno</label>
      <input bind:value={secondFamilyName} class={inputClass} id="patient-second-family-name" type="text" />
    </div>
  </div>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
    <div>
      <label class={labelClass} for="patient-document-type">Documento *</label>
      <select bind:value={documentType} class={inputClass} id="patient-document-type">
        <option value="dni">DNI</option>
        <option value="ce">Carné de extranjería</option>
        <option value="passport">Pasaporte</option>
      </select>
    </div>
    <div class="sm:col-span-2">
      <label class={labelClass} for="patient-document-number">Número de documento *</label>
      <input bind:value={documentNumber} class={inputClass} id="patient-document-number" required type="text" />
    </div>
  </div>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
    <div>
      <label class={labelClass} for="patient-phone">Teléfono *</label>
      <input bind:value={phone} class={inputClass} id="patient-phone" required type="tel" />
    </div>
    <div>
      <label class={labelClass} for="patient-birth-date">Fecha de nacimiento</label>
      <input bind:value={birthDate} class={inputClass} id="patient-birth-date" type="date" />
    </div>
  </div>

  <div class="pt-2 flex justify-end">
    <button
      class="rounded-md bg-cyan-600 px-5 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
      disabled={busy}
      type="submit"
    >
      {busy ? "Registrando..." : "Registrar paciente"}
    </button>
  </div>
</form>
