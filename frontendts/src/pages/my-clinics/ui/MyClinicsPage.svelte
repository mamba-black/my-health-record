<script lang="ts">
import { onMount } from "svelte";
import { loadProfileContext, profileContext } from "$entities/session";
import { runWithRequirements } from "$features/profile";
import {
  ClinicStatus,
  clinicApi,
  type Membership,
  MembershipRole,
  MembershipStatus,
} from "$shared/api";
import { log } from "$shared/lib";

const ROLE_LABELS: Record<MembershipRole, string> = {
  [MembershipRole.UNSPECIFIED]: "Miembro",
  [MembershipRole.OWNER]: "Dueño",
  [MembershipRole.ADMIN]: "Administrador",
  [MembershipRole.PRACTITIONER]: "Médico",
};

const STATUS_LABELS: Record<MembershipStatus, string> = {
  [MembershipStatus.UNSPECIFIED]: "",
  [MembershipStatus.INVITED]: "Invitación pendiente",
  [MembershipStatus.PENDING]: "Pendiente de activación",
  [MembershipStatus.ACTIVE]: "Activo",
  [MembershipStatus.SUSPENDED]: "Suspendido",
};

const CLINIC_STATUS_LABELS: Record<ClinicStatus, string> = {
  [ClinicStatus.UNSPECIFIED]: "",
  [ClinicStatus.PENDING_SETUP]: "Clínica pendiente de activación",
  [ClinicStatus.ACTIVE]: "Clínica activa",
  [ClinicStatus.SUSPENDED]: "Clínica suspendida",
};

interface MembershipAction {
  label: string;
  run: () => Promise<unknown>;
}

let busy = $state<string | null>(null);
let message = $state<string | null>(null);
let errorMessage = $state<string | null>(null);

const memberships = $derived($profileContext?.memberships ?? []);

onMount(() => {
  void loadProfileContext();
});

/** Acciones que la persona puede iniciar sobre cada membresía. */
function actionsFor(membership: Membership): MembershipAction[] {
  const actions: MembershipAction[] = [];
  if (membership.status === MembershipStatus.INVITED) {
    actions.push({
      label: "Unirme",
      run: () => clinicApi.joinClinic(membership.id),
    });
  }
  if (
    membership.role === MembershipRole.PRACTITIONER &&
    (membership.status === MembershipStatus.INVITED ||
      membership.status === MembershipStatus.PENDING)
  ) {
    actions.push({
      label: "Activar mi perfil médico",
      run: () => clinicApi.activatePractitioner(membership.id),
    });
  }
  if (
    membership.role === MembershipRole.OWNER &&
    membership.status === MembershipStatus.ACTIVE &&
    membership.organizationStatus === ClinicStatus.PENDING_SETUP
  ) {
    actions.push({
      label: "Activar clínica",
      run: () => clinicApi.activateClinic(membership.organizationId),
    });
  }
  return actions;
}

async function start(membership: Membership, action: MembershipAction) {
  busy = `${membership.id}:${action.label}`;
  message = null;
  errorMessage = null;
  try {
    const result = await runWithRequirements(action.run);
    if (result !== undefined) {
      message = `Listo: ${action.label.toLowerCase()}.`;
    }
    await loadProfileContext();
  } catch (error) {
    log.error(`No se pudo ${action.label}`, error);
    errorMessage = error instanceof Error ? error.message : "No se pudo completar la acción.";
  } finally {
    busy = null;
  }
}
</script>

<div class="max-w-3xl mx-auto">
  <h1 class="text-2xl font-semibold text-gray-900">Mis clínicas</h1>
  <p class="mt-1 text-sm text-gray-600">Tus invitaciones y tu rol en cada clínica.</p>

  {#if message}
    <div class="mt-4 p-3 rounded-md bg-emerald-50 border border-emerald-200 text-sm text-emerald-800" role="status">
      {message}
    </div>
  {/if}
  {#if errorMessage}
    <div class="mt-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700" role="alert">
      {errorMessage}
    </div>
  {/if}

  {#if memberships.length === 0}
    <p class="mt-6 text-sm text-gray-500">Aún no perteneces a ninguna clínica.</p>
  {:else}
    <ul class="mt-6 divide-y divide-gray-200">
      {#each memberships as membership (membership.id)}
        <li class="py-4 flex flex-wrap items-center justify-between gap-3" data-membership={membership.id}>
          <div>
            <p class="font-medium text-gray-900">{membership.organizationName}</p>
            <p class="text-sm text-gray-600">
              {ROLE_LABELS[membership.role]} · <span data-status>{STATUS_LABELS[membership.status]}</span>
            </p>
            <p class="text-xs text-gray-500">{CLINIC_STATUS_LABELS[membership.organizationStatus]}</p>
          </div>
          <div class="flex gap-2">
            {#each actionsFor(membership) as action (action.label)}
              <button
                class="rounded-md bg-cyan-600 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
                disabled={busy !== null}
                onclick={() => start(membership, action)}
                type="button"
              >
                {action.label}
              </button>
            {/each}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
