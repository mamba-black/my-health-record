<script lang="ts">
import { onMount } from "svelte";
import { goto } from "$app/navigation";
import { setAuthToken, setPendingRegistration } from "$entities/session";
import { UserApiError, userApi } from "$shared/api/user";
import { GOOGLE_CLIENT_ID, SIGNUP } from "$shared/config";
import { decodeGoogleJwt } from "$shared/lib/jwt";
import { log } from "$shared/lib/logger";

interface GoogleCredentialResponse {
  credential?: string;
  select_by?: string;
}

interface GoogleAccountsId {
  initialize: (config: {
    client_id: string;
    callback: (response: GoogleCredentialResponse) => void;
    use_fedcm_for_prompt?: boolean;
    use_fedcm_for_button?: boolean;
  }) => void;
  prompt: () => void;
  renderButton: (
    parent: HTMLElement,
    options: {
      theme?: "outline" | "filled_blue" | "filled_black";
      size?: "large" | "medium" | "small";
      type?: "standard" | "icon";
      text?: "signin_with" | "signup_with" | "continue_with" | "signin";
      shape?: "rectangular" | "pill" | "circle" | "square";
    },
  ) => void;
}

type WindowWithGoogle = Window & {
  google?: {
    accounts?: {
      id?: GoogleAccountsId;
    };
  };
};

const getGoogleId = (): GoogleAccountsId | undefined =>
  (window as WindowWithGoogle).google?.accounts?.id;

let email = $state("");
let password = $state("");
let googleButtonContainer: HTMLDivElement | undefined = $state();
let gisLoaded = $state(false);
let isSubmitting = $state(false);
let errorMessage = $state<string | null>(null);

const handleGoogleCredentialResponse = async (
  response: GoogleCredentialResponse,
) => {
  if (!response.credential) {
    log.error("No credential received from Google GIS", response);
    errorMessage = "No se pudo obtener la credencial de Google.";
    return;
  }

  const idToken = response.credential;
  const claims = decodeGoogleJwt(idToken);
  log.info("Google GIS/FedCM JWT credential received", claims?.email);

  errorMessage = null;
  isSubmitting = true;

  try {
    const result = await userApi.signIn({
      idToken,
      providerId: "google.com",
    });

    const displayName =
      result.givenName || claims?.given_name || claims?.name || result.email;
    setAuthToken(idToken, displayName);
  } catch (error) {
    if (error instanceof UserApiError && error.code === "NOT_FOUND") {
      log.info(
        "Usuario no encontrado en la red clínica; redirigiendo a /signup para perfilado progresivo",
      );
      setPendingRegistration({
        idToken,
        email: claims?.email ?? "",
        givenName: claims?.given_name ?? claims?.name ?? "",
        familyName: claims?.family_name,
        picture: claims?.picture,
        providerId: "google.com",
      });
      await goto(SIGNUP);
      return;
    }

    log.error("Error durante la validación de usuario con Google:", error);
    errorMessage =
      error instanceof Error
        ? error.message
        : "No se pudo validar tu cuenta con el servidor.";
  } finally {
    isSubmitting = false;
  }
};

onMount(() => {
  const initGoogleGis = () => {
    const googleId = getGoogleId();
    if (!googleId) return;

    googleId.initialize({
      client_id: GOOGLE_CLIENT_ID,
      callback: handleGoogleCredentialResponse,
      use_fedcm_for_prompt: true,
      use_fedcm_for_button: true,
    });

    if (googleButtonContainer) {
      googleId.renderButton(googleButtonContainer, {
        theme: "outline",
        size: "large",
        text: "continue_with",
        shape: "rectangular",
      });
    }
    gisLoaded = true;
  };

  if (getGoogleId()) {
    initGoogleGis();
    return;
  }

  const script = document.createElement("script");
  script.src = "https://accounts.google.com/gsi/client";
  script.async = true;
  script.defer = true;
  script.onload = initGoogleGis;
  script.onerror = () => {
    log.error("Failed to load Google Identity Services script");
  };
  document.head.appendChild(script);
});

const handleGooglePromptFallback = () => {
  const googleId = getGoogleId();
  if (googleId) {
    googleId.prompt();
  } else {
    errorMessage =
      "El servicio de Google aún no está disponible o fue bloqueado por el navegador.";
  }
};

const loginHandler = (event: Event) => {
  event.preventDefault();
  errorMessage = null;
  setAuthToken("1234567890", email.trim() || "Miuler");
};
</script>

<div class="max-w-md mx-auto">
  <div>
    <h1 class="text-2xl font-semibold">Login</h1>
  </div>

  {#if errorMessage}
    <div class="mt-4 p-3 rounded-md bg-red-50 border border-red-200 text-sm text-red-700">
      {errorMessage}
    </div>
  {/if}

  {#if isSubmitting}
    <div class="mt-4 p-3 rounded-md bg-cyan-50 border border-cyan-200 text-sm text-cyan-800">
      Verificando tu cuenta con Google...
    </div>
  {/if}

  <div class="divide-y divide-gray-200">
    <form onsubmit={loginHandler}>
      <div class="py-8 text-base leading-6 space-y-4 text-gray-700 sm:text-lg sm:leading-7">
        <div class="relative">
          <input
            bind:value={email}
            autocomplete="email"
            class="peer placeholder-transparent h-10 w-full border-b-2 border-gray-300 text-gray-900 focus:outline-none focus:border-rose-600"
            id="email"
            name="email"
            placeholder="Email address"
            type="text"
          />
          <label
            class="absolute left-0 -top-3.5 text-gray-600 text-sm peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-440 peer-placeholder-shown:top-2 transition-all peer-focus:-top-3.5 peer-focus:text-gray-600 peer-focus:text-sm"
            for="email"
          >Email Address</label>
        </div>
        <div class="relative">
          <input
            bind:value={password}
            autocomplete="current-password"
            class="peer placeholder-transparent h-10 w-full border-b-2 border-gray-300 text-gray-900 focus:outline-none focus:border-rose-600"
            id="password"
            name="password"
            placeholder="Password"
            type="password"
          />
          <label
            class="absolute left-0 -top-3.5 text-gray-600 text-sm peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-440 peer-placeholder-shown:top-2 transition-all peer-focus:-top-3.5 peer-focus:text-gray-600 peer-focus:text-sm"
            for="password"
          >Password</label>
        </div>
        <div class="relative">
          <button
            class="bg-cyan-500 hover:bg-cyan-600 disabled:opacity-50 text-white rounded-md px-4 py-1.5"
            disabled={isSubmitting}
            type="submit"
          >
            Submit
          </button>
        </div>
      </div>
    </form>
  </div>
</div>

<div class="w-full flex flex-col items-center justify-center">
  <div bind:this={googleButtonContainer} class="flex justify-center"></div>

  {#if !gisLoaded}
    <button
      onclick={handleGooglePromptFallback}
      type="button"
      class="flex items-center bg-white border border-gray-300 rounded-lg shadow-md px-6 py-2 text-sm font-medium text-gray-800 hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
    >
      <span>Continue with Google</span>
    </button>
  {/if}
</div>
