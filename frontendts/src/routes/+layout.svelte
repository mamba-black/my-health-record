<script lang="ts">
import type { Snippet } from "svelte";
import { AppLayout } from "$app_layer/ui";

let { children }: { children: Snippet } = $props();
</script>

<AppLayout>
  {@render children()}
</AppLayout>
