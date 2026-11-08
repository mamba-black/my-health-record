<script lang="ts">
import { onMount } from "svelte";
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { formatDateTime } from "$entities/appointment";
import { authStore } from "$entities/session";
import {
  SlotPicker,
  savePendingBooking,
  takePendingBooking,
} from "$features/book-appointment";
import { runWithRequirements } from "$features/profile";
import {
  type AppointmentInfo,
  ClinicStatus,
  type PublicClinic,
  type PublicPractitioner,
  publicSchedulingApi,
  type SlotInfo,
  schedulingApi,
} from "$shared/api";
import { LOGIN, MY_APPOINTMENTS, Page } from "$shared/config";
import { log, rememberClinicLink } from "$shared/lib";

let { subdomain }: { subdomain: string } = $props();

let clinic = $state<PublicClinic | null>(null);
let practitioners = $state<PublicPractitioner[]>([]);
let selected = $state<string | null>(null);
let loading = $state(true);
let busy = $state(false);
let errorMessage = $state<string | null>(null);
let booked = $state<AppointmentInfo | null>(null);
let picker = $state<{ reload: () => Promise<void> } | null>(null);

const timezone = $derived(clinic?.timezone || "America/Lima");

async function book(practitionerUserId: string, start: string) {
  if (!clinic) return;
  if (!$authStore.isAuthenticated) {
    // Sin sesión: se guarda el cupo y se vuelve aquí tras iniciar sesión.
    savePendingBooking({ subdomain, organizationId: clinic.organizationId, practitionerUserId, start });
    await goto(LOGIN, { state: new Page(page.url.pathname) });
    return;
  }
  busy = true;
  errorMessage = null;
  try {
    const organizationId = clinic.organizationId;
    const result = await runWithRequirements(() =>
      schedulingApi.book(organizationId, practitionerUserId, start),
    );
    if (result) booked = result;
  } catch (error) {
    log.error("No se pudo reservar", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo reservar la cita.";
    await picker?.reload();
  } finally {
    busy = false;
  }
}

function pick(slot: SlotInfo) {
  if (selected) void book(selected, slot.start);
}

onMount(async () => {
  rememberClinicLink(subdomain);
  try {
    clinic = await publicSchedulingApi.getClinic(subdomain);
    practitioners = await publicSchedulingApi.listPractitioners(subdomain);
    selected = practitioners[0]?.userId ?? null;
  } catch (error) {
    log.error("No se pudo abrir la clínica", error);
    errorMessage = "No encontramos una clínica con ese enlace.";
  } finally {
    loading = false;
  }
  const pending = takePendingBooking(subdomain);
  if (pending && $authStore.isAuthenticated) {
    selected = pending.practitionerUserId;
    await book(pending.practitionerUserId, pending.start);
  }
});
</script>

<div class="max-w-3xl mx-auto">
  {#if loading}
    <p class="text-sm text-gray-500">Cargando...</p>
  {:else if !clinic}
    <p class="text-sm text-red-700" role="alert">{errorMessage}</p>
  {:else}
    <h1 class="text-2xl font-semibold text-gray-900">{clinic.name}</h1>
    {#if clinic.address}<p class="text-sm text-gray-600">{clinic.address}</p>{/if}

    {#if clinic.status !== ClinicStatus.ACTIVE}
      <p class="mt-6 p-3 rounded-md bg-amber-50 border border-amber-200 text-sm text-amber-800" role="status">
        Esta clínica aún no atiende. Vuelve pronto.
      </p>
    {:else if booked}
      <div class="mt-6 p-4 rounded-md bg-emerald-50 border border-emerald-200 text-emerald-900" role="status">
        <p class="font-medium">Tu cita quedó confirmada.</p>
        <p class="text-sm">
          {formatDateTime(booked.start, timezone)} con {booked.practitionerName}.
        </p>
        <a class="mt-2 inline-block text-sm font-medium underline" href={MY_APPOINTMENTS}>Ver mis citas</a>
      </div>
    {:else if practitioners.length === 0}
      <p class="mt-6 text-sm text-gray-500">Por ahora no hay médicos con horario publicado.</p>
    {:else}
      {#if errorMessage}
        <p class="mt-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700" role="alert">
          {errorMessage}
        </p>
      {/if}
      <h2 class="mt-6 text-lg font-semibold text-gray-900">Reserva tu cita</h2>
      <div class="mt-2 flex flex-wrap gap-2" role="tablist">
        {#each practitioners as practitioner (practitioner.userId)}
          <button
            aria-selected={selected === practitioner.userId}
            class="rounded-md border px-3 py-2 text-left text-sm {selected === practitioner.userId
              ? 'border-cyan-600 bg-cyan-50'
              : 'border-gray-300'}"
            onclick={() => (selected = practitioner.userId)}
            role="tab"
            type="button"
          >
            <span class="block font-medium text-gray-900">{practitioner.name}</span>
            {#if practitioner.specialty}
              <span class="block text-xs text-gray-600">{practitioner.specialty}</span>
            {/if}
          </button>
        {/each}
      </div>
      {#if selected}
        <SlotPicker
          bind:this={picker}
          {busy}
          onPick={pick}
          practitionerUserId={selected}
          {subdomain}
          {timezone}
        />
      {/if}
    {/if}
  {/if}
</div>
