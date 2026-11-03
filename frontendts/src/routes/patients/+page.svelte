<script lang="ts">
import { goto } from "$app/navigation";
import { SEARCH } from "$shared/config";
import { onMount } from "svelte";

onMount(() => {
  goto(SEARCH);
});
</script>
