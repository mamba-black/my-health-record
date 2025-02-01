<script lang="ts">
  let { label, onclick }: { label: string; onclick?: () => void } = $props();
  let _type: "reset" | undefined = $state("reset");
  if (onclick != undefined) {
    _type = undefined;
  }
</script>

<button type={_type} class="button-reset" onclick={onclick}>
  {label}
</button>
