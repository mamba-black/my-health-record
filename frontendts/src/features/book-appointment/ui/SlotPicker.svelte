<script lang="ts">
import { dayKey, formatDay, formatTime } from "$entities/appointment";
import { publicSchedulingApi, type SlotInfo } from "$shared/api";
import { log } from "$shared/lib";

/**
 * Cupos libres de un médico en los próximos días, agrupados por día. Al elegir uno
 * llama a `onPick`.
 */
let {
  subdomain,
  practitionerUserId,
  timezone,
  days = 7,
  busy = false,
  onPick,
}: {
  subdomain: string;
  practitionerUserId: string;
  timezone: string;
  days?: number;
  busy?: boolean;
  onPick: (slot: SlotInfo) => void;
} = $props();

let slots = $state<SlotInfo[]>([]);
let loading = $state(true);
let errorMessage = $state<string | null>(null);

const groups = $derived(
  slots.reduce<{ key: string; label: string; slots: SlotInfo[] }[]>((acc, slot) => {
    const key = dayKey(slot.start, timezone);
    const group = acc.find((g) => g.key === key);
    if (group) group.slots.push(slot);
    else acc.push({ key, label: formatDay(slot.start, timezone), slots: [slot] });
    return acc;
  }, []),
);

export async function reload() {
  loading = true;
  errorMessage = null;
  try {
    const from = new Date();
    const to = new Date(from.getTime() + days * 24 * 60 * 60 * 1000);
    slots = await publicSchedulingApi.getAvailability(subdomain, practitionerUserId, from, to);
  } catch (error) {
    log.error("No se pudieron leer los cupos", error);
    errorMessage = "No se pudieron leer los cupos libres.";
  } finally {
    loading = false;
  }
}

$effect(() => {
  void practitionerUserId;
  void reload();
});
</script>

<div data-testid="slot-picker">
  {#if loading}
    <p class="text-sm text-gray-500">Buscando cupos libres...</p>
  {:else if errorMessage}
    <p class="text-sm text-red-700" role="alert">{errorMessage}</p>
  {:else if groups.length === 0}
    <p class="text-sm text-gray-500">No hay cupos libres en los próximos {days} días.</p>
  {:else}
    {#each groups as group (group.key)}
      <div class="mt-3">
        <p class="text-sm font-medium text-gray-800 capitalize">{group.label}</p>
        <div class="mt-1 flex flex-wrap gap-2">
          {#each group.slots as slot (slot.id)}
            <button
              class="rounded-md border border-cyan-600 px-3 py-1 text-sm text-cyan-700 hover:bg-cyan-50 disabled:opacity-50"
              data-slot={slot.start}
              disabled={busy}
              onclick={() => onPick(slot)}
              type="button"
            >
              {formatTime(slot.start, timezone)}
            </button>
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>
