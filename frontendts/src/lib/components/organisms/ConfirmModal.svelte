<script lang="ts">
  // Declara la prop `isOpen` para controlar la visibilidad del diálogo
  // Usa $props para props reactivas en Svelte 5+
  type PROPS = {
    isOpen: boolean,
    message?: string,
    callback?: (confirm: boolean) => void,
  };
  let {
    isOpen = $bindable(false),
    message = "Estás editando informacion del usuario",
    callback,
  }: PROPS = $props();

  // Declara un evento `on:confirm` que se despachará cuando el usuario confirme
  // Declara un evento `on:cancel` que se despachará cuando el usuario cancele
  // const { confirm, cancel } = $event();

  function handleConfirm() {
    isOpen = false; // Cierra el diálogo
    callback?.(true); // Emite el evento de confirmación
  }

  function handleCancel() {
    isOpen = false; // Cierra el diálogo
    callback?.(false); // Emite el evento de cancelación
  }

  // Previene que se cierre el diálogo al hacer clic fuera de su contenido principal
  function handleBackdropClick(event: MouseEvent) {
    // Si el clic fue directamente en el fondo (backdrop), cierra el diálogo.
    // Esto es útil si el diálogo contiene elementos clicables que no deberían cerrarlo.
    if (event.target === event.currentTarget) {
      // Puedes elegir si quieres que un clic en el fondo cancele o no.
      // Para este ejemplo, lo dejaremos para que solo los botones lo hagan.
      // Si quieres que el fondo cierre, descomenta la siguiente línea:
      // handleCancel();
    }
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 bg-gray-400/50 overflow-y-auto h-full w-full flex items-center justify-center z-50"
    onclick={handleBackdropClick}
  >
    <div class="relative p-8 bg-white w-96 max-w-md mx-auto rounded-lg shadow-xl">
      <!-- Título y mensaje -->
      <p class="text-lg leading-6 font-medium text-gray-900 mb-4">{message}</p>
      <!--      <div class="mt-2 mb-6">-->
      <!--        <p class="text-sm text-gray-500">-->
      <!--          {message}-->
      <!--        </p>-->
      <!--      </div>-->

      <!-- Botones de acción -->
      <div class="flex justify-end gap-x-4">
        <button
          type="button"
          onclick={handleConfirm}
          class="button-reset"
        >
          Continuar Editando
        </button>
      </div>
    </div>
  </div>
{/if}
