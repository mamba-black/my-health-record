<script lang="ts">
import { onMount } from "svelte";
import { MEMBERSHIP_ROLE_LABELS, MEMBERSHIP_STATUS_LABELS } from "$entities/profile";
import { type ClinicMember, clinicApi, MembershipRole, MembershipStatus } from "$shared/api";
import { log } from "$shared/lib";

/**
 * Personal de una clínica para el dueño o un administrador: aprueba o rechaza las
 * solicitudes de ingreso e invita por correo.
 */
let { organizationId }: { organizationId: string } = $props();

let members = $state<ClinicMember[]>([]);
let loading = $state(true);
let busy = $state(false);
let errorMessage = $state<string | null>(null);
let inviteEmail = $state("");
let inviteRole = $state<MembershipRole>(MembershipRole.RECEPTIONIST);
let inviteMessage = $state<string | null>(null);

const requests = $derived(
  members.filter((member) => member.membership?.status === MembershipStatus.REQUESTED),
);
const others = $derived(
  members.filter((member) => member.membership?.status !== MembershipStatus.REQUESTED),
);

async function load() {
  loading = true;
  errorMessage = null;
  try {
    members = await clinicApi.listClinicMembers(organizationId);
  } catch (error) {
    log.error("No se pudo leer el personal", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo leer el personal.";
  } finally {
    loading = false;
  }
}

async function act(run: () => Promise<unknown>) {
  busy = true;
  errorMessage = null;
  try {
    await run();
    await load();
  } catch (error) {
    log.error("No se pudo completar la acción sobre el personal", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo completar la acción.";
  } finally {
    busy = false;
  }
}

async function invite(event: Event) {
  event.preventDefault();
  inviteMessage = null;
  const email = inviteEmail.trim();
  if (!email) return;
  await act(async () => {
    await clinicApi.inviteMember(organizationId, email, inviteRole);
    inviteMessage = `Invitaste a ${email}. Verá la invitación en «Mis clínicas».`;
    inviteEmail = "";
  });
}

onMount(load);
</script>

<section class="mt-3 rounded-lg border border-gray-200 p-4" data-testid="staff-panel">
  {#if errorMessage}
    <p class="mb-2 text-sm text-red-700" role="alert">{errorMessage}</p>
  {/if}

  <h3 class="text-sm font-semibold text-gray-900">Solicitudes de ingreso</h3>
  {#if loading}
    <p class="text-sm text-gray-500">Cargando...</p>
  {:else if requests.length === 0}
    <p class="text-sm text-gray-500">No hay solicitudes pendientes.</p>
  {:else}
    <ul class="mt-2 divide-y divide-gray-100">
      {#each requests as member (member.membership?.id)}
        <li class="py-2 flex flex-wrap items-center justify-between gap-2" data-request={member.email}>
          <div>
            <p class="text-sm font-medium text-gray-900">{member.displayName || member.email}</p>
            <p class="text-xs text-gray-600">
              {member.email} · quiere entrar como {MEMBERSHIP_ROLE_LABELS[member.membership?.role ?? MembershipRole.UNSPECIFIED].toLowerCase()}
            </p>
          </div>
          <div class="flex gap-2">
            <button
              class="rounded-md bg-cyan-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
              disabled={busy}
              onclick={() => act(() => clinicApi.approveMembership(member.membership?.id ?? ""))}
              type="button"
            >
              Aprobar
            </button>
            <button
              class="rounded-md border border-gray-300 px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50"
              disabled={busy}
              onclick={() => act(() => clinicApi.suspendMembership(member.membership?.id ?? ""))}
              type="button"
            >
              Rechazar
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <h3 class="mt-4 text-sm font-semibold text-gray-900">Personal</h3>
  {#if !loading}
    <ul class="mt-2 divide-y divide-gray-100">
      {#each others as member (member.membership?.id)}
        <li class="py-2 text-sm text-gray-700">
          <span class="font-medium text-gray-900">{member.displayName || member.email}</span>
          · {MEMBERSHIP_ROLE_LABELS[member.membership?.role ?? MembershipRole.UNSPECIFIED]}
          · {MEMBERSHIP_STATUS_LABELS[member.membership?.status ?? MembershipStatus.UNSPECIFIED]}
        </li>
      {/each}
    </ul>
  {/if}

  <h3 class="mt-4 text-sm font-semibold text-gray-900">Invitar por correo</h3>
  <form class="mt-2 flex flex-wrap items-end gap-3" onsubmit={invite}>
    <input
      bind:value={inviteEmail}
      aria-label="Correo de la persona a invitar"
      class="rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
      placeholder="correo@ejemplo.com"
      type="email"
    />
    <select
      bind:value={inviteRole}
      aria-label="Rol de la persona invitada"
      class="rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
    >
      <option value={MembershipRole.RECEPTIONIST}>Recepcionista</option>
      <option value={MembershipRole.PRACTITIONER}>Médico</option>
      <option value={MembershipRole.ADMIN}>Administrador</option>
    </select>
    <button
      class="rounded-md bg-cyan-600 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
      disabled={busy}
      type="submit"
    >
      Invitar
    </button>
  </form>
  {#if inviteMessage}
    <p class="mt-2 text-sm text-emerald-700" role="status">{inviteMessage}</p>
  {/if}
</section>
