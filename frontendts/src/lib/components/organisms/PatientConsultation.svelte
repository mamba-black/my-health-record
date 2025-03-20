<script lang="ts">
  import PatientDetail from "$lib/components/organisms/PatientDetail.svelte";

  let { isOpen = $bindable(), patient } = $props();
  // let { patient }: { patient: Patient } = $props();
</script>

{#if isOpen}
  <div class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-4/5 max-w-[800px] bg-white p-6 rounded-lg shadow-xl">
    <PatientDetail patient={patient}>

    </PatientDetail>
  </div>
{/if}
