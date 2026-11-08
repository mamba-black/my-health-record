<script lang="ts">
import { RegisterPatientForm } from "$features/register-patient";
import type { Patient } from "$shared/api";
import { HISTORY } from "$shared/config";

let registered = $state<Patient | null>(null);

const nameOf = (patient: Patient) =>
  [...(patient.name[0]?.given ?? []), ...(patient.name[0]?.family ?? [])].join(" ");
</script>

<div class="max-w-2xl mx-auto">
  <h1 class="text-2xl font-semibold text-gray-900">Registrar paciente</h1>
  <p class="mt-1 mb-6 text-sm text-gray-600">
    Para pacientes sin cuenta, por ejemplo quien llama por teléfono. Si luego crea su cuenta,
    la recepción la vincula con este registro mostrando su documento.
  </p>

  {#if registered}
    <div class="mb-6 p-3 rounded-md bg-emerald-50 border border-emerald-200 text-sm text-emerald-800" role="status">
      Registraste a {nameOf(registered)}.
      <a class="font-medium underline" href="{HISTORY}/{registered.id}">Ver su ficha</a>
    </div>
  {/if}

  <RegisterPatientForm onRegistered={(patient) => (registered = patient)} />
</div>
