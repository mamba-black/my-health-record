<script lang="ts">
import { goto } from "$app/navigation";
import {
  authStore,
  clearPendingRegistration,
  setAuthToken,
} from "$entities/session";
import { UserApiError } from "$shared/api";
import { ROOT } from "$shared/config";
import { log } from "$shared/lib";
import { completeProfile } from "../api/auth.api";

const pending = $authStore.pendingRegistration;

let givenName = $state(pending?.givenName ?? "");
let familyName = $state(pending?.familyName ?? "");
let secondFamilyName = $state(pending?.secondFamilyName ?? "");
let dni = $state(pending?.dni ?? "");
let phone = $state(pending?.phone ?? "");
let birthDate = $state(pending?.birthDate ?? "");
let address = $state(pending?.address ?? "");
let isSubmitting = $state(false);
let errorMessage = $state<string | null>(null);

const handleSkip = async () => {
  clearPendingRegistration();
  await goto(ROOT);
};

const handleSubmit = async (event: Event) => {
  event.preventDefault();
  errorMessage = null;

  const trimmedGivenName = givenName.trim();
  const trimmedPhone = phone.trim();
  const trimmedBirthDate = birthDate.trim();
  const trimmedDni = dni.trim();
  if (!trimmedGivenName || !trimmedPhone || !trimmedBirthDate) {
    errorMessage =
      "Por favor completa los campos obligatorios (nombres, teléfono y fecha de nacimiento).";
    return;
  }

  isSubmitting = true;
  try {
    const idToken = $authStore.token ?? "";
    const account = await completeProfile({
      idToken,
      givenName: trimmedGivenName,
      familyName: familyName.trim() || undefined,
      secondFamilyName: secondFamilyName.trim() || undefined,
      identifier: trimmedDni
        ? { identifierType: { oneofKind: "dni", dni: trimmedDni } }
        : undefined,
      phone: trimmedPhone,
      birthDate: trimmedBirthDate,
      address: address.trim() || undefined,
    });
    setAuthToken(idToken, account.givenName);
  } catch (error) {
    log.error("Error al completar los datos de la cuenta:", error);
    if (error instanceof UserApiError && error.code === "ALREADY_EXISTS") {
      errorMessage = "El DNI ingresado ya está registrado en otra cuenta.";
    } else {
      errorMessage =
        error instanceof Error ? error.message : "No se pudieron guardar tus datos.";
    }
  } finally {
    isSubmitting = false;
  }
};
</script>

<div class="max-w-lg mx-auto">
  <div class="mb-6 flex items-center gap-4">
    {#if pending?.picture}
      <img
        alt="Avatar de Google"
        class="w-12 h-12 rounded-full border border-gray-200"
        src={pending.picture}
      />
    {/if}
    <div>
      <h1 class="text-2xl font-semibold text-gray-900">Completa tus datos</h1>
      <p class="text-sm text-gray-600">
        Registra los datos obligatorios de tu cuenta.
      </p>
    </div>
  </div>

  {#if errorMessage}
    <div class="mb-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700">
      {errorMessage}
    </div>
  {/if}

  <form class="space-y-4" onsubmit={handleSubmit}>
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-email">
        Correo electrónico
      </label>
      <input
        class="w-full rounded-md border border-gray-300 bg-gray-50 px-3 py-2 text-gray-500"
        disabled
        id="profile-email"
        type="email"
        value={pending?.email ?? ""}
      />
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-given-name">
          Nombres *
        </label>
        <input
          bind:value={givenName}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="profile-given-name"
          maxlength="100"
          required
          type="text"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-family-name">
          Primer apellido
        </label>
        <input
          bind:value={familyName}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="profile-family-name"
          maxlength="50"
          type="text"
        />
      </div>
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-second-family-name">
          Segundo apellido (opcional)
        </label>
        <input
          bind:value={secondFamilyName}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="profile-second-family-name"
          maxlength="50"
          type="text"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-dni">
          DNI (opcional)
        </label>
        <input
          bind:value={dni}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="profile-dni"
          inputmode="numeric"
          maxlength="8"
          placeholder="8 dígitos"
          type="text"
        />
      </div>
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-phone">
          Teléfono de contacto *
        </label>
        <input
          bind:value={phone}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="profile-phone"
          maxlength="20"
          placeholder="+51 999 888 777"
          required
          type="tel"
        />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-birth-date">
          Fecha de nacimiento *
        </label>
        <input
          bind:value={birthDate}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="profile-birth-date"
          required
          type="date"
        />
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1" for="profile-address">
        Dirección (opcional)
      </label>
      <input
        bind:value={address}
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
        id="profile-address"
        maxlength="255"
        type="text"
      />
    </div>

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
</div>
