<script lang="ts">
let { label = "" }: { label?: string } = $props();
</script>

<button class="button-submit" type="submit">
  {label}
</button>
