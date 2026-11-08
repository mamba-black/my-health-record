<script lang="ts">
import { onMount } from "svelte";
import {
  APPOINTMENT_STATUS_LABELS,
  DEFAULT_TIMEZONE,
  formatDateTime,
  isUpcoming,
} from "$entities/appointment";
import { SlotPicker } from "$features/book-appointment";
import { type AppointmentInfo, AppointmentStatus, type SlotInfo, schedulingApi } from "$shared/api";
import { log } from "$shared/lib";

let appointments = $state<AppointmentInfo[]>([]);
let loading = $state(true);
let busy = $state(false);
let message = $state<string | null>(null);
let errorMessage = $state<string | null>(null);
/** Cita que se está reprogramando o por cancelar. */
let moving = $state<string | null>(null);
let confirmingCancel = $state<string | null>(null);

const changeable = (a: AppointmentInfo) =>
  a.status === AppointmentStatus.BOOKED && isUpcoming(a.start);

async function load() {
  loading = true;
  try {
    appointments = (await schedulingApi.myAppointments()).sort((a, b) =>
      b.start.localeCompare(a.start),
    );
  } catch (error) {
    log.error("No se pudieron leer las citas", error);
    errorMessage = "No se pudieron leer tus citas.";
  } finally {
    loading = false;
  }
}

async function act(run: () => Promise<AppointmentInfo>, done: string) {
  busy = true;
  message = null;
  errorMessage = null;
  try {
    await run();
    message = done;
    moving = null;
    confirmingCancel = null;
    await load();
  } catch (error) {
    log.error("No se pudo cambiar la cita", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo cambiar la cita.";
  } finally {
    busy = false;
  }
}

const reschedule = (a: AppointmentInfo, slot: SlotInfo) =>
  act(
    () => schedulingApi.reschedule(a.id, a.practitionerUserId, slot.start),
    `Tu cita pasó al ${formatDateTime(slot.start, DEFAULT_TIMEZONE)}.`,
  );

onMount(load);
</script>

<div class="max-w-3xl mx-auto">
  <h1 class="text-2xl font-semibold text-gray-900">Mis citas</h1>
  {#if message}
    <p class="mt-4 p-3 rounded-md bg-emerald-50 border border-emerald-200 text-sm text-emerald-800" role="status">{message}</p>
  {/if}
  {#if errorMessage}
    <p class="mt-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700" role="alert">{errorMessage}</p>
  {/if}

  {#if loading}
    <p class="mt-6 text-sm text-gray-500">Cargando...</p>
  {:else if appointments.length === 0}
    <p class="mt-6 text-sm text-gray-500">Aún no tienes citas. Resérvalas desde el enlace de tu clínica.</p>
  {:else}
    <ul class="mt-6 divide-y divide-gray-200">
      {#each appointments as a (a.id)}
        <li class="py-4" data-appointment={a.id}>
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p class="font-medium text-gray-900 capitalize">{formatDateTime(a.start)}</p>
              <p class="text-sm text-gray-600">{a.practitionerName} · {a.clinicName}</p>
              <p class="text-xs text-gray-500" data-status>{APPOINTMENT_STATUS_LABELS[a.status]}</p>
            </div>
            {#if changeable(a)}
              <div class="flex gap-2">
                <button
                  class="rounded-md border border-gray-300 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
                  disabled={busy}
                  onclick={() => (moving = moving === a.id ? null : a.id)}
                  type="button"
                >
                  Cambiar horario
                </button>
                {#if confirmingCancel === a.id}
                  <button
                    class="rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-red-700"
                    disabled={busy}
                    onclick={() => act(() => schedulingApi.cancel(a.id), "Cancelaste tu cita.")}
                    type="button"
                  >
                    Sí, cancelar
                  </button>
                {:else}
                  <button
                    class="rounded-md border border-red-300 px-3 py-1.5 text-sm text-red-700 hover:bg-red-50"
                    disabled={busy}
                    onclick={() => (confirmingCancel = a.id)}
                    type="button"
                  >
                    Cancelar
                  </button>
                {/if}
              </div>
            {/if}
          </div>
          {#if moving === a.id}
            <div class="mt-3 rounded-md border border-gray-200 p-3">
              <p class="text-sm text-gray-700">Elige el nuevo horario con {a.practitionerName}:</p>
              <SlotPicker
                {busy}
                onPick={(slot) => reschedule(a, slot)}
                practitionerUserId={a.practitionerUserId}
                subdomain={a.clinicSubdomain}
                timezone={DEFAULT_TIMEZONE}
              />
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>
