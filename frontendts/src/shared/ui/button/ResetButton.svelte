<script lang="ts">
let { label, onclick }: { label: string; onclick?: () => void } = $props();
const buttonType = $derived(onclick !== undefined ? ("button" as const) : ("reset" as const));
</script>

<button class="button-reset" onclick={onclick} type={buttonType}>
  {label}
</button>
