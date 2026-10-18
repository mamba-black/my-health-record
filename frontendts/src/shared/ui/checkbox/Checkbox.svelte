<script lang="ts">
let {
  id,
  name,
  readonly,
  value = $bindable(),
}: {
  id: string;
  name: string;
  value: boolean;
  readonly: boolean;
} = $props();
</script>

{#if !readonly || value}
  <div class="history-checkbox flex items-center">
    <input
      id={id}
      disabled={readonly}
      bind:checked={value}
      type="checkbox"
      class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
    />

    <label for={id} class="ms-2">
      {name}
    </label>
  </div>
{/if}
