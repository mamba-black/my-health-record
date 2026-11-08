<script lang="ts">
import { onMount } from "svelte";
import { formatDateTime } from "$entities/appointment";
import { SlotPicker } from "$features/book-appointment";
import {
  type AppointmentInfo,
  type PublicPractitioner,
  publicSchedulingApi,
  type SlotInfo,
  schedulingApi,
} from "$shared/api";
import { log } from "$shared/lib";

/**
 * Recepción reserva en nombre de un paciente que llamó por teléfono, aunque no tenga
 * cuenta. Un médico solo reserva en su propia agenda (`onlyPractitioner`).
 */
let {
  organizationId,
  subdomain,
  timezone,
  onlyPractitioner = null,
  onBooked,
}: {
  organizationId: string;
  subdomain: string;
  timezone: string;
  onlyPractitioner?: string | null;
  onBooked?: (appointment: AppointmentInfo) => void;
} = $props();

let practitioners = $state<PublicPractitioner[]>([]);
let practitioner = $state<string>("");
let givenName = $state("");
let familyName = $state("");
let dni = $state("");
let phone = $state("");
let busy = $state(false);
let errorMessage = $state<string | null>(null);
let booked = $state<AppointmentInfo | null>(null);

async function book(slot: SlotInfo) {
  errorMessage = null;
  if (!givenName.trim() || !familyName.trim() || !dni.trim() || !phone.trim()) {
    errorMessage = "Completa nombres, apellido paterno, DNI y teléfono del paciente.";
    return;
  }
  busy = true;
  try {
    booked = await schedulingApi.bookForPatient({
      organizationId,
      practitionerUserId: practitioner,
      start: slot.start,
      patient: {
        oneofKind: "newPatient",
        newPatient: {
          givenName: givenName.trim(),
          familyName: familyName.trim(),
          identifier: { identifierType: { oneofKind: "dni", dni: dni.trim() } },
          phone: phone.trim(),
        },
      },
    });
    givenName = familyName = dni = phone = "";
    onBooked?.(booked);
  } catch (error) {
    log.error("No se pudo reservar por el paciente", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo reservar.";
  } finally {
    busy = false;
  }
}

onMount(async () => {
  practitioners = (await publicSchedulingApi.listPractitioners(subdomain)).filter(
    (p) => !onlyPractitioner || p.userId === onlyPractitioner,
  );
  practitioner = practitioners[0]?.userId ?? "";
});
</script>

<section class="rounded-lg border border-gray-200 p-4" data-testid="book-for-patient">
  <h2 class="text-lg font-semibold text-gray-900">Reservar por un paciente</h2>
  {#if booked}
    <p class="mt-2 text-sm text-emerald-700" role="status">
      Reservaste para {booked.patientName} el {formatDateTime(booked.start, timezone)}.
    </p>
  {/if}
  {#if errorMessage}
    <p class="mt-2 text-sm text-red-700" role="alert">{errorMessage}</p>
  {/if}
  <div class="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
    <input aria-label="Nombres del paciente" bind:value={givenName} class="rounded border border-gray-300 px-2 py-1" placeholder="Nombres" />
    <input aria-label="Apellido paterno del paciente" bind:value={familyName} class="rounded border border-gray-300 px-2 py-1" placeholder="Apellido paterno" />
    <input aria-label="DNI del paciente" bind:value={dni} class="rounded border border-gray-300 px-2 py-1" placeholder="DNI" />
    <input aria-label="Teléfono del paciente" bind:value={phone} class="rounded border border-gray-300 px-2 py-1" placeholder="Teléfono" />
  </div>
  {#if practitioners.length === 0}
    <p class="mt-3 text-sm text-gray-500">No hay médicos con horario publicado.</p>
  {:else}
    <select aria-label="Médico" bind:value={practitioner} class="mt-3 rounded border border-gray-300 px-2 py-1">
      {#each practitioners as p (p.userId)}
        <option value={p.userId}>{p.name}</option>
      {/each}
    </select>
    {#if practitioner}
      {#key practitioner}
        <SlotPicker {busy} onPick={book} practitionerUserId={practitioner} {subdomain} {timezone} />
      {/key}
    {/if}
  {/if}
</section>
