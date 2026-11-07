<script lang="ts">
import { onMount } from "svelte";
import { goto } from "$app/navigation";
import {
  authStore,
  clearPendingRegistration,
  loadProfileContext,
  profileContext,
  setSessionUser,
} from "$entities/session";
import { UserApiError } from "$shared/api";
import { ROOT } from "$shared/config";
import { log } from "$shared/lib";
import { saveProfile } from "../api/save-profile";
import { draftFromAccount, type FieldErrors, validateDraft } from "../model/draft";
import { groupFields, requiredForMap } from "../model/groups";
import ProfileFields from "./ProfileFields.svelte";

let draft = $state(draftFromAccount(undefined));
let loaded = $state(false);
let isSubmitting = $state(false);
let errors = $state<FieldErrors>({});
let errorMessage = $state<string | null>(null);

const groups = $derived(
  groupFields($profileContext?.fields.map((requirement) => requirement.field) ?? []),
);
const requiredFor = $derived(requiredForMap($profileContext?.fields ?? []));
const readOnly = $derived(
  ($profileContext?.fields ?? [])
    .filter((requirement) => requirement.readOnly)
    .map((requirement) => requirement.field),
);

onMount(async () => {
  const context = await loadProfileContext();
  const loadedDraft = draftFromAccount(context?.account);
  // Lo que Google ya conoce precarga los datos que la cuenta aún no tiene.
  const google = $authStore.pendingRegistration;
  if (google) {
    loadedDraft.givenName ||= google.givenName;
    loadedDraft.familyName ||= google.familyName ?? "";
  }
  draft = loadedDraft;
  loaded = true;
});

const handleSkip = async () => {
  clearPendingRegistration();
  await goto(ROOT);
};

// Ningún dato es obligatorio al guardar: solo se valida el formato.
const handleSubmit = async (event: Event) => {
  event.preventDefault();
  errorMessage = null;
  errors = validateDraft(draft, []);
  if (Object.keys(errors).length > 0) return;

  isSubmitting = true;
  try {
    const account = await saveProfile(draft);
    clearPendingRegistration();
    setSessionUser(account.givenName || account.email);
    await loadProfileContext();
    await goto(ROOT);
  } catch (error) {
    log.error("Error al guardar los datos de la cuenta:", error);
    errorMessage =
      error instanceof UserApiError && error.code === "ALREADY_EXISTS"
        ? "El documento ingresado ya está registrado en otra cuenta."
        : error instanceof Error
          ? error.message
          : "No se pudieron guardar tus datos.";
  } finally {
    isSubmitting = false;
  }
};
</script>

<div class="max-w-2xl mx-auto">
  <div class="mb-6">
    <h1 class="text-2xl font-semibold text-gray-900">Completa tus datos</h1>
    <p class="text-sm text-gray-600">
      Ningún dato es obligatorio ahora: te pediremos cada uno cuando vayas a hacer algo que lo necesite.
    </p>
  </div>

  {#if errorMessage}
    <div class="mb-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700">
      {errorMessage}
    </div>
  {/if}

  {#if loaded}
    <form class="space-y-6" onsubmit={handleSubmit}>
      {#each groups as group (group.title)}
        <section>
          <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">
            {group.title}
          </h2>
          <ProfileFields bind:draft {errors} fields={group.fields} {readOnly} {requiredFor} />
        </section>
      {/each}

      <div class="pt-4 flex items-center justify-end gap-3">
        <button
          class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
          onclick={handleSkip}
          type="button"
        >
          Ahora no
        </button>
        <button
          class="rounded-md bg-cyan-600 px-5 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
          disabled={isSubmitting}
          type="submit"
        >
          {isSubmitting ? "Guardando..." : "Guardar datos"}
        </button>
      </div>
    </form>
  {:else}
    <p class="text-sm text-gray-500">Cargando tus datos...</p>
  {/if}
</div>
