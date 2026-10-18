<script lang="ts">
type Props = {
  isOpen: boolean;
  message?: string;
  callback?: (confirm: boolean) => void;
};

let {
  isOpen = $bindable(false),
  message = "Estás editando informacion del usuario",
  callback,
}: Props = $props();

function handleConfirm() {
  isOpen = false;
  callback?.(true);
}
</script>

{#if isOpen}
  <div
    class="fixed inset-0 bg-gray-400/50 overflow-y-auto h-full w-full flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
  >
    <div class="relative p-8 bg-white w-96 max-w-md mx-auto rounded-lg shadow-xl">
      <p class="text-lg leading-6 font-medium text-gray-900 mb-4">{message}</p>

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
