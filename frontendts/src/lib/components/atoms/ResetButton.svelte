<script lang="ts">
  let {label, onclick}: { label: string; onclick?: () => void } = $props();
  let _type: "reset" | undefined = $state("reset");
  if (onclick != undefined) {
    _type = undefined;
  }
</script>

<button class="button-reset" onclick={onclick} type={_type}>
  {label}
</button>
