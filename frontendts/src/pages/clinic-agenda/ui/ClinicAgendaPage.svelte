<script lang="ts">
import { onMount } from "svelte";
import {
  APPOINTMENT_STATUS_LABELS,
  DEFAULT_TIMEZONE,
  formatTime,
  isUpcoming,
} from "$entities/appointment";
import { agendaClinics, loadProfileContext, profileContext } from "$entities/session";
import { BookForPatientForm } from "$features/reception";
import {
  type AppointmentInfo,
  AppointmentStatus,
  DocumentCheck,
  type LinkSuggestionInfo,
  MembershipRole,
  MembershipStatus,
  schedulingApi,
} from "$shared/api";
import { log } from "$shared/lib";

let selected = $state<string | null>(null);
let day = $state(new Date().toLocaleDateString("en-CA"));
let appointments = $state<AppointmentInfo[]>([]);
let busy = $state(false);
let message = $state<string | null>(null);
let errorMessage = $state<string | null>(null);
let checks = $state<Record<string, DocumentCheck>>({});
let suggestions = $state<{ appointment: AppointmentInfo; accounts: LinkSuggestionInfo[] } | null>(null);

const clinic = $derived($agendaClinics.find((m) => m.organizationId === selected));
const isReception = $derived(
  ($profileContext?.memberships ?? []).some(
    (m) =>
      m.organizationId === selected &&
      m.status === MembershipStatus.ACTIVE &&
      [MembershipRole.OWNER, MembershipRole.ADMIN, MembershipRole.RECEPTIONIST].includes(m.role),
  ),
);

$effect(() => {
  if (!selected && $agendaClinics.length > 0) selected = $agendaClinics[0].organizationId;
});

async function load() {
  if (!selected) return;
  const from = new Date(`${day}T00:00:00`);
  const to = new Date(from.getTime() + 24 * 60 * 60 * 1000);
  try {
    appointments = await schedulingApi.clinicAppointments(selected, from, to);
  } catch (error) {
    log.error("No se pudo leer la agenda", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo leer la agenda.";
  }
}

$effect(() => {
  void selected;
  void day;
  void load();
});

async function act<T>(run: () => Promise<T>, done: string): Promise<T | undefined> {
  busy = true;
  message = null;
  errorMessage = null;
  try {
    const result = await run();
    message = done;
    await load();
    return result;
  } catch (error) {
    log.error("No se pudo completar la acción", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo completar la acción.";
    return undefined;
  } finally {
    busy = false;
  }
}

async function checkIn(a: AppointmentInfo) {
  const check = checks[a.id] ?? DocumentCheck.UNSPECIFIED;
  const result = await act(() => schedulingApi.checkIn(a.id, check), "Registraste la llegada.");
  if (!result) return;
  if (result.accountVerified) message = "Registraste la llegada y el documento quedó verificado.";
  if (check === DocumentCheck.MISMATCH) {
    message = "Registraste la llegada. El documento no coincide: corrige los datos del paciente.";
  }
  suggestions = result.linkSuggestions.length > 0 ? { appointment: a, accounts: result.linkSuggestions } : null;
}

async function link(account: LinkSuggestionInfo) {
  if (!suggestions || !selected) return;
  const { appointment } = suggestions;
  const organizationId = selected;
  await act(
    () => schedulingApi.linkAccount(organizationId, appointment.patientId, account.userId),
    `Vinculaste al paciente con la cuenta de ${account.name}.`,
  );
  suggestions = null;
}

onMount(() => {
  void loadProfileContext();
});
</script>

<div class="max-w-4xl mx-auto">
  <h1 class="text-2xl font-semibold text-gray-900">Agenda de la clínica</h1>
  {#if $agendaClinics.length === 0}
    <p class="mt-4 text-sm text-gray-500">No tienes una membresía activa en una clínica activa.</p>
  {:else}
    <div class="mt-3 flex flex-wrap items-center gap-3">
      {#if $agendaClinics.length > 1}
        <select aria-label="Clínica" bind:value={selected} class="rounded-md border border-gray-300 px-2 py-1">
          {#each $agendaClinics as m (m.organizationId)}
            <option value={m.organizationId}>{m.organizationName}</option>
          {/each}
        </select>
      {:else}
        <span class="text-sm text-gray-700">{clinic?.organizationName}</span>
      {/if}
      <input aria-label="Día" bind:value={day} class="rounded-md border border-gray-300 px-2 py-1" type="date" />
    </div>

    {#if message}
      <p class="mt-4 p-3 rounded-md bg-emerald-50 border border-emerald-200 text-sm text-emerald-800" role="status">{message}</p>
    {/if}
    {#if errorMessage}
      <p class="mt-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700" role="alert">{errorMessage}</p>
    {/if}

    {#if suggestions}
      <div class="mt-4 rounded-md border border-cyan-200 bg-cyan-50 p-3" data-testid="link-suggestions">
        <p class="text-sm text-gray-800">
          {suggestions.appointment.patientName} no tiene cuenta, pero hay cuentas con el mismo documento.
          Vincúlala solo si viste el documento físico del paciente.
        </p>
        {#each suggestions.accounts as account (account.userId)}
          <div class="mt-2 flex items-center gap-3 text-sm">
            <span>{account.name} ({account.email})</span>
            <button class="rounded-md bg-cyan-600 px-3 py-1 text-white hover:bg-cyan-700" disabled={busy} onclick={() => link(account)} type="button">
              Vi el documento físico: vincular
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <ul class="mt-6 divide-y divide-gray-200">
      {#each appointments as a (a.id)}
        <li class="py-3 flex flex-wrap items-center justify-between gap-3" data-appointment={a.id}>
          <div>
            <p class="font-medium text-gray-900">{formatTime(a.start, DEFAULT_TIMEZONE)} · {a.patientName ?? "Paciente"}</p>
            <p class="text-sm text-gray-600">{a.practitionerName} · <span data-status>{APPOINTMENT_STATUS_LABELS[a.status]}</span></p>
          </div>
          {#if isReception && a.status === AppointmentStatus.BOOKED}
            <div class="flex flex-wrap items-center gap-2">
              <select
                aria-label="Documento del paciente"
                class="rounded border border-gray-300 px-2 py-1 text-sm"
                onchange={(e) => (checks[a.id] = Number((e.target as HTMLSelectElement).value))}
              >
                <option value={DocumentCheck.UNSPECIFIED}>Sin verificar el documento</option>
                <option value={DocumentCheck.MATCH}>Vi el documento físico y coincide</option>
                <option value={DocumentCheck.MISMATCH}>El documento no coincide</option>
              </select>
              <button class="rounded-md bg-cyan-600 px-3 py-1 text-sm text-white hover:bg-cyan-700" disabled={busy} onclick={() => checkIn(a)} type="button">
                Llegó
              </button>
              {#if !isUpcoming(a.start)}
                <button class="rounded-md border border-gray-300 px-3 py-1 text-sm" disabled={busy} onclick={() => act(() => schedulingApi.markNoShow(a.id), "Marcaste la inasistencia.")} type="button">
                  No asistió
                </button>
              {:else}
                <button class="rounded-md border border-red-300 px-3 py-1 text-sm text-red-700" disabled={busy} onclick={() => act(() => schedulingApi.cancel(a.id), "Cancelaste la cita.")} type="button">
                  Cancelar
                </button>
              {/if}
            </div>
          {/if}
        </li>
      {:else}
        <li class="py-3 text-sm text-gray-500">No hay citas este día.</li>
      {/each}
    </ul>

    {#if clinic}
      <div class="mt-8">
        {#key clinic.organizationId}
          <BookForPatientForm
            onBooked={() => void load()}
            onlyPractitioner={isReception ? null : ($profileContext?.memberships.find((m) => m.organizationId === selected)?.userId ?? null)}
            organizationId={clinic.organizationId}
            subdomain={clinic.organizationSubdomain}
            timezone={DEFAULT_TIMEZONE}
          />
        {/key}
      </div>
    {/if}
  {/if}
</div>
