<script lang="ts">
import { goto } from "$app/navigation";
import {
  authStore,
  clearPendingRegistration,
  setAuthToken,
} from "$entities/session";
import { SignUpIntent, UserApiError, userApi } from "$shared/api/user";
import { LOGIN } from "$shared/config/routes";
import { generateUuidV7 } from "$shared/lib/jwt";
import { log } from "$shared/lib/logger";

const pending = $derived($authStore.pendingRegistration);

let email = $state($authStore.pendingRegistration?.email ?? "");
let givenName = $state($authStore.pendingRegistration?.givenName ?? "");
let familyName = $state($authStore.pendingRegistration?.familyName ?? "");
let secondFamilyName = $state("");
let phone = $state("");
let birthDate = $state("");
let address = $state("");
let dni = $state("");
let selectedIntent = $state<SignUpIntent>(SignUpIntent.INTENT_PATIENT);
let isSubmitting = $state(false);
let errorMessage = $state<string | null>(null);

$effect(() => {
  if (pending) {
    if (!email && pending.email) email = pending.email;
    if (!givenName && pending.givenName) givenName = pending.givenName;
    if (!familyName && pending.familyName) familyName = pending.familyName;
  }
});

const handleCancel = async () => {
  clearPendingRegistration();
  await goto(LOGIN);
};

const handleSignUpSubmit = async (event: Event) => {
  event.preventDefault();
  errorMessage = null;

  const trimmedEmail = email.trim();
  const trimmedGivenName = givenName.trim();
  const trimmedPhone = phone.trim();
  const trimmedBirthDate = birthDate.trim();
  const trimmedDni = dni.trim();

  if (!trimmedEmail || !trimmedGivenName || !trimmedPhone || !trimmedBirthDate) {
    errorMessage =
      "Por favor completa los campos obligatorios (nombres, correo, teléfono y fecha de nacimiento).";
    return;
  }

  isSubmitting = true;
  const idToken = pending?.idToken ?? "local-dev-token";
  const providerId = pending?.providerId ?? "google.com";
  const userId = generateUuidV7();

  try {
    const response = await userApi.signUp({
      idToken,
      userId,
      providerId,
      providerName: "Google",
      providerAvatarUrl: pending?.picture,
      email: trimmedEmail,
      identifier:
        trimmedDni.length > 0
          ? {
              identifierType: {
                oneofKind: "dni",
                dni: trimmedDni,
              },
            }
          : undefined,
      givenName: trimmedGivenName,
      familyName: familyName.trim() || undefined,
      secondFamilyName: secondFamilyName.trim() || undefined,
      phone: trimmedPhone,
      address: address.trim(),
      birthDate: trimmedBirthDate,
      displayName: `${trimmedGivenName} ${familyName.trim()}`.trim(),
      createClinic: selectedIntent === SignUpIntent.INTENT_CLINIC_OWNER,
      intent: selectedIntent,
    });

    log.info("Registro progresivo completado exitosamente:", response.userId);
    setAuthToken(idToken, trimmedGivenName);
  } catch (error) {
    log.error("Error al registrar usuario:", error);
    if (error instanceof UserApiError && error.code === "ALREADY_EXISTS") {
      errorMessage =
        "El correo o documento ingresado ya se encuentra registrado en esta red clínica.";
    } else {
      errorMessage =
        error instanceof Error
          ? error.message
          : "No se pudo completar el registro.";
    }
  } finally {
    isSubmitting = false;
  }
};
</script>

<div class="max-w-lg mx-auto">
  <div class="flex items-center gap-4 mb-6">
    {#if pending?.picture}
      <img
        alt="Avatar de Google"
        class="w-12 h-12 rounded-full border border-gray-200"
        src={pending.picture}
      />
    {/if}
    <div>
      <h1 class="text-2xl font-semibold text-gray-900">Completa tu Registro</h1>
      <p class="text-sm text-gray-600">
        Valida tus datos para activar tu cuenta en la plataforma.
      </p>
    </div>
  </div>

  {#if errorMessage}
    <div class="mb-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700">
      {errorMessage}
    </div>
  {/if}

  <form class="space-y-4" onsubmit={handleSignUpSubmit}>
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-intent">
        ¿Cómo usarás la plataforma? *
      </label>
      <select
        bind:value={selectedIntent}
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
        id="signup-intent"
      >
        <option value={SignUpIntent.INTENT_PATIENT}>Paciente</option>
        <option value={SignUpIntent.INTENT_PRACTITIONER}>Profesional de Salud (Médico / Especialista)</option>
        <option value={SignUpIntent.INTENT_CLINIC_OWNER}>Administrador / Fundador de Clínica</option>
      </select>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-email">
        Correo electrónico *
      </label>
      <input
        bind:value={email}
        class="w-full rounded-md border border-gray-300 bg-gray-50 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
        id="signup-email"
        required
        type="email"
      />
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-given-name">
          Nombres *
        </label>
        <input
          bind:value={givenName}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="signup-given-name"
          required
          type="text"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-family-name">
          Primer apellido
        </label>
        <input
          bind:value={familyName}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="signup-family-name"
          type="text"
        />
      </div>
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-second-family-name">
          Segundo apellido (opcional)
        </label>
        <input
          bind:value={secondFamilyName}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="signup-second-family-name"
          type="text"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-dni">
          DNI (opcional en registro inicial)
        </label>
        <input
          bind:value={dni}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="signup-dni"
          maxlength="8"
          placeholder="8 dígitos"
          type="text"
        />
      </div>
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-phone">
          Teléfono de contacto *
        </label>
        <input
          bind:value={phone}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="signup-phone"
          placeholder="+51 999 888 777"
          required
          type="tel"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-birth-date">
          Fecha de nacimiento *
        </label>
        <input
          bind:value={birthDate}
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
          id="signup-birth-date"
          required
          type="date"
        />
      </div>
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-1" for="signup-address">
        Dirección (opcional)
      </label>
      <input
        bind:value={address}
        class="w-full rounded-md border border-gray-300 px-3 py-2 text-gray-900 focus:border-cyan-600 focus:outline-none"
        id="signup-address"
        type="text"
      />
    </div>

    <div class="pt-4 flex items-center justify-end gap-3">
      <button
        class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
        onclick={handleCancel}
        type="button"
      >
        Volver al Login
      </button>
      <button
        class="rounded-md bg-cyan-600 px-5 py-2 text-sm font-medium text-white hover:bg-cyan-700 disabled:opacity-50"
        disabled={isSubmitting}
        type="submit"
      >
        {isSubmitting ? "Registrando..." : "Crear cuenta"}
      </button>
    </div>
  </form>
</div>
