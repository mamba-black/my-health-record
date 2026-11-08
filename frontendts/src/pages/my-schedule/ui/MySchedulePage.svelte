<script lang="ts">
import { onMount } from "svelte";
import { APPOINTMENT_STATUS_LABELS, formatDateTime } from "$entities/appointment";
import { loadProfileContext, practitionerClinics } from "$entities/session";
import { ScheduleEditor } from "$features/manage-schedule";
import { type AppointmentInfo, schedulingApi } from "$shared/api";
import { CLINIC_PUBLIC_PREFIX } from "$shared/config";
import { log } from "$shared/lib";

let selected = $state<string | null>(null);
let appointments = $state<AppointmentInfo[]>([]);

const clinic = $derived($practitionerClinics.find((m) => m.organizationId === selected));

$effect(() => {
  if (!selected && $practitionerClinics.length > 0) selected = $practitionerClinics[0].organizationId;
});

$effect(() => {
  const organizationId = selected;
  if (!organizationId) return;
  const from = new Date();
  const to = new Date(from.getTime() + 7 * 24 * 60 * 60 * 1000);
  schedulingApi
    .clinicAppointments(organizationId, from, to)
    .then((list) => (appointments = list))
    .catch((error) => log.warn("No se pudo leer la agenda", error));
});

onMount(() => {
  void loadProfileContext();
});
</script>

<div class="max-w-3xl mx-auto">
  <h1 class="text-2xl font-semibold text-gray-900">Mi agenda</h1>
  {#if $practitionerClinics.length === 0}
    <p class="mt-4 text-sm text-gray-500">Necesitas una membresía de médico activa en una clínica activa.</p>
  {:else}
    {#if $practitionerClinics.length > 1}
      <select bind:value={selected} class="mt-3 rounded-md border border-gray-300 px-2 py-1" aria-label="Clínica">
        {#each $practitionerClinics as m (m.organizationId)}
          <option value={m.organizationId}>{m.organizationName}</option>
        {/each}
      </select>
    {/if}
    {#if clinic}
      <p class="mt-2 text-sm text-gray-600">
        {clinic.organizationName} · enlace público:
        <a class="underline" href="{CLINIC_PUBLIC_PREFIX}{clinic.organizationSubdomain}">/c/{clinic.organizationSubdomain}</a>
      </p>
      {#key clinic.organizationId}
        <ScheduleEditor organizationId={clinic.organizationId} />
      {/key}

      <h2 class="mt-8 text-lg font-semibold text-gray-900">Mis citas de los próximos días</h2>
      {#if appointments.length === 0}
        <p class="text-sm text-gray-500">No tienes citas reservadas.</p>
      {:else}
        <ul class="mt-2 divide-y divide-gray-100">
          {#each appointments as a (a.id)}
            <li class="py-2 text-sm text-gray-800">
              <span class="capitalize">{formatDateTime(a.start)}</span> · {a.patientName ?? "Paciente"} ·
              <span class="text-gray-500">{APPOINTMENT_STATUS_LABELS[a.status]}</span>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  {/if}
</div>
