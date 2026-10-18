<script lang="ts">
import type { Snippet } from "svelte";

let { children }: { children?: Snippet } = $props();
</script>

<h2 class="section-title col-span-3 text-xl font-semibold text-gray-700 mb-4 pb-2 border-b border-gray-200">
  {@render children?.()}
</h2>
