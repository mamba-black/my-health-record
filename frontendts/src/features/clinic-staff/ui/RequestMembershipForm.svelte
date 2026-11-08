<script lang="ts">
import { onMount } from "svelte";
import { loadProfileContext } from "$entities/session";
import { runWithRequirements } from "$features/profile";
import { clinicApi, MembershipRole } from "$shared/api";
import { log } from "$shared/lib";

/**
 * Pedir entrar a una clínica existente, por su enlace, como médico o recepcionista.
 * La solicitud queda pendiente hasta que el dueño o un administrador la apruebe.
 */
let {
  initialSubdomain = "",
  initialRole = MembershipRole.RECEPTIONIST,
  autoSubmit = false,
}: {
  initialSubdomain?: string;
  initialRole?: MembershipRole;
  autoSubmit?: boolean;
} = $props();

let subdomain = $state(initialSubdomain);
let role = $state<MembershipRole>(initialRole);
let busy = $state(false);
let message = $state<string | null>(null);
let errorMessage = $state<string | null>(null);

async function submit(event?: Event) {
  event?.preventDefault();
  if (!subdomain.trim()) {
    errorMessage = "Indica el enlace de la clínica.";
    return;
  }
  busy = true;
  message = null;
  errorMessage = null;
  try {
    const result = await runWithRequirements(() =>
      clinicApi.requestMembership(subdomain.trim(), role),
    );
    if (result) {
      message = `Enviaste tu solicitud a ${result.organizationName}. Te avisaremos aquí cuando la aprueben.`;
      subdomain = "";
      await loadProfileContext();
    }
  } catch (error) {
    log.error("No se pudo pedir el ingreso", error);
    errorMessage = error instanceof Error ? error.message : "No se pudo enviar la solicitud.";
  } finally {
    busy = false;
  }
}

onMount(() => {
  if (autoSubmit && initialSubdomain) void submit();
});
</script>

<form class="mt-3 flex flex-wrap items-end gap-3" data-testid="request-membership" onsubmit={submit}>
  <div>
    <label class="block text-sm font-medium text-gray-700 mb-1" for="request-clinic">Enlace de la clínica</label>
    <input
      bind:value={subdomain}
      class="rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
      id="request-clinic"
      placeholder="san-borja"
      type="text"
    />
  </div>
  <div>
    <label class="block text-sm font-medium text-gray-700 mb-1" for="request-role">Como</label>
    <select
      bind:value={role}
      class="rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
      id="request-role"
    >
      <option value={MembershipRole.RECEPTIONIST}>Recepcionista</option>
      <option value={MembershipRole.PRACTITIONER}>Médico</option>
    </select>
  </div>
  <button
    class="rounded-md bg-cyan-600 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
    disabled={busy}
    type="submit"
  >
    {busy ? "Enviando..." : "Pedir entrar"}
  </button>
</form>
{#if message}
  <p class="mt-2 text-sm text-emerald-700" role="status">{message}</p>
{/if}
{#if errorMessage}
  <p class="mt-2 text-sm text-red-700" role="alert">{errorMessage}</p>
{/if}
