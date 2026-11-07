<script lang="ts">
import { tick } from "svelte";
import { loadProfileContext, profileContext } from "$entities/session";
import { ACTION_PURPOSE, fieldList } from "$entities/profile";
import { MissingDataError, ProfileField, toMissingDataError } from "$shared/api";
import { log } from "$shared/lib";
import { saveProfile } from "../api/save-profile";
import {
  type FieldErrors,
  NON_FILLABLE_FIELDS,
  type ProfileDraft,
  draftFromAccount,
  validateDraft,
} from "../model/draft";
import { groupFields, requiredForMap } from "../model/groups";
import { pendingRequirement } from "../model/requirements";
import ProfileFields from "./ProfileFields.svelte";

let draft = $state<ProfileDraft>(draftFromAccount(undefined));
let view = $state<"reduced" | "full">("reduced");
let confirmingExit = $state(false);
let isSaving = $state(false);
let errors = $state<FieldErrors>({});
let errorMessage = $state<string | null>(null);
let dialog: HTMLDivElement | undefined = $state();
let openedFor: unknown = null;

const pending = $derived($pendingRequirement);
const purpose = $derived(pending ? ACTION_PURPOSE[pending.action] : "");
const fillable = $derived(
  (pending?.missing ?? []).filter((field) => !NON_FILLABLE_FIELDS.has(field)),
);
const blockedByClinic = $derived(
  (pending?.missing ?? []).includes(ProfileField.LICENSE_CONFIRMED),
);
const fullGroups = $derived(
  groupFields(
    [
      ...($profileContext?.fields.map((requirement) => requirement.field) ?? []),
      ...(pending?.missing ?? []),
    ],
    { includeClinic: !!pending?.organizationId },
  ),
);
const requiredFor = $derived(requiredForMap($profileContext?.fields ?? []));
const readOnly = $derived(
  ($profileContext?.fields ?? [])
    .filter((requirement) => requirement.readOnly)
    .map((requirement) => requirement.field),
);

// Al abrir la ventana: borrador con los valores actuales y vista reducida.
$effect(() => {
  if (!pending || openedFor === pending) return;
  openedFor = pending;
  view = "reduced";
  confirmingExit = false;
  errors = {};
  errorMessage = null;
  draft = draftFromAccount($profileContext?.account);
  void loadProfileContext().then((context) => {
    if (context?.account && openedFor === pending) {
      // Conserva lo que el usuario ya haya escrito.
      const fresh = draftFromAccount(context.account);
      for (const key of Object.keys(fresh) as (keyof ProfileDraft)[]) {
        if (!draft[key]) (draft as unknown as Record<string, unknown>)[key] = fresh[key];
      }
    }
  });
  void tick().then(() => dialog?.querySelector<HTMLElement>("input, select, button")?.focus());
});

const askToExit = () => {
  confirmingExit = true;
};

const exitWithoutContinuing = () => {
  const current = pending;
  openedFor = null;
  draft = draftFromAccount(undefined);
  current?.finish(false);
};

const complete = async () => {
  if (!pending) return;
  errorMessage = null;
  errors = validateDraft(draft, pending.missing);
  if (Object.keys(errors).length > 0) return;

  isSaving = true;
  try {
    await saveProfile(draft, { organizationId: pending.organizationId });
    await pending.retry();
    await loadProfileContext();
    openedFor = null;
    pending.finish(true);
  } catch (error) {
    const missing = error instanceof MissingDataError ? error : toMissingDataError(error);
    if (missing) {
      // La acción aún pide datos: se queda abierta con la lista actualizada.
      pendingRequirement.set({ ...pending, missing: missing.missing });
      openedFor = $pendingRequirement;
      errors = Object.fromEntries(
        missing.missing.map((field) => [field, "Este dato es obligatorio para continuar"]),
      );
    } else {
      log.error("No se pudieron guardar los datos de la ventana", error);
      errorMessage = error instanceof Error ? error.message : "No se pudieron guardar tus datos.";
    }
  } finally {
    isSaving = false;
  }
};

const onKeydown = (event: KeyboardEvent) => {
  if (pending && event.key === "Escape") {
    event.preventDefault();
    if (!confirmingExit) askToExit();
  }
};
</script>

<svelte:window onkeydown={onKeydown} />

{#if pending}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4"
    data-testid="requirements-backdrop"
    onclick={(event) => {
      if (event.target === event.currentTarget && !confirmingExit) askToExit();
    }}
  >
    <div
      aria-labelledby="requirements-title"
      aria-modal="true"
      bind:this={dialog}
      class="w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-2xl bg-white p-6 shadow-lg"
      role="dialog"
    >
      {#if confirmingExit}
        <h2 class="text-lg font-semibold text-gray-900" id="requirements-title">
          ¿Salir sin completar?
        </h2>
        <p class="mt-3 p-3 rounded-md bg-amber-50 border border-amber-200 text-sm text-amber-800">
          Sin tu {fieldList(pending.missing)} no podrás {purpose}. Si sales, perderás los datos que ingresaste.
        </p>
        <div class="pt-5 flex items-center justify-end gap-3">
          <button
            class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
            onclick={exitWithoutContinuing}
            type="button"
          >
            Salir sin continuar
          </button>
          <button
            class="rounded-md bg-cyan-600 px-5 py-2 text-sm font-medium text-white hover:bg-cyan-700"
            onclick={() => (confirmingExit = false)}
            type="button"
          >
            Volver a completar
          </button>
        </div>
      {:else}
        <h2 class="text-lg font-semibold text-gray-900" id="requirements-title">
          Completa tus datos
        </h2>
        <p class="mt-1 text-sm text-gray-600">
          Para {purpose} necesitamos estos datos.
        </p>

        {#if errorMessage}
          <div class="mt-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700">
            {errorMessage}
          </div>
        {/if}

        <form
          class="mt-4 space-y-6"
          onsubmit={(event) => {
            event.preventDefault();
            void complete();
          }}
        >
          {#if view === "reduced"}
            <ProfileFields
              bind:draft
              {errors}
              fields={[...fillable, ...(blockedByClinic ? [ProfileField.LICENSE_CONFIRMED] : [])]}
              idPrefix="requirement"
              required={pending.missing}
              {readOnly}
              {requiredFor}
            />
          {:else}
            {#each fullGroups as group (group.title)}
              <section>
                <h3 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">
                  {group.title}
                </h3>
                <ProfileFields
                  bind:draft
                  {errors}
                  fields={group.fields}
                  idPrefix="requirement"
                  required={pending.missing}
                  {readOnly}
                  {requiredFor}
                />
              </section>
            {/each}
          {/if}

          <div class="flex flex-wrap items-center justify-between gap-3 pt-2">
            <button
              class="text-sm font-medium text-cyan-700 hover:underline"
              onclick={() => (view = view === "reduced" ? "full" : "reduced")}
              type="button"
            >
              {view === "reduced" ? "Editar todos mis datos" : "Ver solo lo necesario"}
            </button>
            <div class="flex items-center gap-3">
              <button
                class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
                onclick={askToExit}
                type="button"
              >
                Rechazar
              </button>
              <button
                class="rounded-md bg-cyan-600 px-5 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
                disabled={isSaving}
                type="submit"
              >
                {isSaving ? "Guardando..." : "Completar"}
              </button>
            </div>
          </div>
        </form>
      {/if}
    </div>
  </div>
{/if}
