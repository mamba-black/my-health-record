<script lang="ts">
import { onMount } from "svelte";
import { formatDateTime } from "$entities/appointment";
import { runWithRequirements } from "$features/profile";
import { type ScheduleResponse, schedulingApi, type TimeBlockInfo } from "$shared/api";
import { log } from "$shared/lib";

/** Editor del horario semanal y de las excepciones del médico en una clínica. */
let { organizationId }: { organizationId: string } = $props();

const DAYS = ["Lunes", "Martes", "Miércoles", "Jueves", "Viernes", "Sábado", "Domingo"];

let slotMinutes = $state(30);
let blocks = $state<TimeBlockInfo[]>([]);
let schedule = $state<ScheduleResponse | null>(null);
let busy = $state(false);
let message = $state<string | null>(null);
let errorMessage = $state<string | null>(null);
let exceptionStart = $state("");
let exceptionEnd = $state("");
let exceptionReason = $state("");

function apply(response: ScheduleResponse) {
  schedule = response;
  slotMinutes = response.slotMinutes || 30;
  blocks = response.blocks.map((b) => ({ ...b }));
}

async function run(action: () => Promise<ScheduleResponse>, done: string) {
  busy = true;
  message = null;
  errorMessage = null;
  try {
    const response = await runWithRequirements(action);
    if (response) {
      apply(response);
      message = done;
    }
  } catch (error) {
    log.error("No se pudo guardar la agenda", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo guardar.";
  } finally {
    busy = false;
  }
}

const addBlock = (weekday: number) =>
  (blocks = [...blocks, { weekday, start: "09:00", end: "13:00" }]);
const removeBlock = (block: TimeBlockInfo) => (blocks = blocks.filter((b) => b !== block));

const save = () =>
  run(
    () => schedulingApi.setWeeklySchedule({ organizationId, slotMinutes, blocks }),
    "Guardaste tu horario.",
  );

function addException(event: Event) {
  event.preventDefault();
  if (!exceptionStart || !exceptionEnd) return;
  const start = new Date(exceptionStart);
  const end = new Date(exceptionEnd);
  void run(
    () => schedulingApi.addException(organizationId, start, end, exceptionReason || undefined),
    "Registraste la ausencia.",
  ).then(() => {
    exceptionStart = exceptionEnd = exceptionReason = "";
  });
}

onMount(async () => {
  try {
    apply(await schedulingApi.getSchedule(organizationId));
  } catch (error) {
    log.error("No se pudo leer el horario", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo leer tu horario.";
  }
});
</script>

<section data-testid="schedule-editor">
  {#if message}
    <p class="mt-2 p-3 rounded-md bg-emerald-50 border border-emerald-200 text-sm text-emerald-800" role="status">{message}</p>
  {/if}
  {#if errorMessage}
    <p class="mt-2 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700" role="alert">{errorMessage}</p>
  {/if}

  <h2 class="mt-4 text-lg font-semibold text-gray-900">Horario semanal</h2>
  <label class="mt-2 block text-sm text-gray-700">
    Duración de cada cita
    <select bind:value={slotMinutes} class="ml-2 rounded-md border border-gray-300 px-2 py-1">
      {#each [15, 20, 30, 45, 60] as minutes (minutes)}
        <option value={minutes}>{minutes} minutos</option>
      {/each}
    </select>
  </label>

  <div class="mt-3 divide-y divide-gray-100">
    {#each DAYS as day, index (day)}
      {@const weekday = index + 1}
      <div class="py-2 flex flex-wrap items-center gap-3" data-weekday={weekday}>
        <span class="w-24 text-sm font-medium text-gray-800">{day}</span>
        {#each blocks.filter((b) => b.weekday === weekday) as block (block)}
          <span class="flex items-center gap-1 text-sm">
            <input aria-label="{day} desde" bind:value={block.start} class="rounded border border-gray-300 px-1" type="time" />
            –
            <input aria-label="{day} hasta" bind:value={block.end} class="rounded border border-gray-300 px-1" type="time" />
            <button aria-label="Quitar franja del {day}" class="text-gray-500 hover:text-red-600" onclick={() => removeBlock(block)} type="button">×</button>
          </span>
        {/each}
        <button class="text-sm text-cyan-700 hover:underline" onclick={() => addBlock(weekday)} type="button">
          + franja
        </button>
      </div>
    {/each}
  </div>
  <button
    class="mt-3 rounded-md bg-cyan-600 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
    disabled={busy}
    onclick={save}
    type="button"
  >
    Guardar horario
  </button>

  <h2 class="mt-8 text-lg font-semibold text-gray-900">Ausencias</h2>
  <p class="text-sm text-gray-600">Feriados, vacaciones u otras ausencias: esos días no se ofrecen cupos.</p>
  <form class="mt-2 flex flex-wrap items-end gap-3" onsubmit={addException}>
    <label class="text-sm text-gray-700">Desde
      <input bind:value={exceptionStart} class="block rounded border border-gray-300 px-2 py-1" type="datetime-local" />
    </label>
    <label class="text-sm text-gray-700">Hasta
      <input bind:value={exceptionEnd} class="block rounded border border-gray-300 px-2 py-1" type="datetime-local" />
    </label>
    <label class="text-sm text-gray-700">Motivo
      <input bind:value={exceptionReason} class="block rounded border border-gray-300 px-2 py-1" placeholder="Vacaciones" type="text" />
    </label>
    <button class="rounded-md border border-cyan-600 px-3 py-1.5 text-sm text-cyan-700 hover:bg-cyan-50 disabled:opacity-50" disabled={busy} type="submit">
      Agregar ausencia
    </button>
  </form>
  <ul class="mt-3 space-y-1">
    {#each schedule?.exceptions ?? [] as exception (exception.id)}
      <li class="flex items-center gap-3 text-sm text-gray-700">
        <span class="capitalize">{formatDateTime(exception.start, schedule?.timezone)} → {formatDateTime(exception.end, schedule?.timezone)}</span>
        {#if exception.reason}<span class="text-gray-500">({exception.reason})</span>{/if}
        <button
          class="text-red-700 hover:underline"
          disabled={busy}
          onclick={() => run(() => schedulingApi.removeException(organizationId, exception.id), "Quitaste la ausencia.")}
          type="button"
        >
          Quitar
        </button>
      </li>
    {/each}
  </ul>
</section>
