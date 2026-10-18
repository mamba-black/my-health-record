<script lang="ts">
import { goto } from "$app/navigation";
import { SEARCH } from "$shared/config/routes";
import { onMount } from "svelte";

onMount(() => {
  goto(SEARCH);
});
</script>
