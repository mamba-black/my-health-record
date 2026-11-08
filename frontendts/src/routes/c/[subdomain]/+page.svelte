<script lang="ts">
import { page } from "$app/state";
import { ClinicPublicPage } from "$pages/clinic-public";
</script>

{#key page.params.subdomain}
  <ClinicPublicPage subdomain={page.params.subdomain ?? ""} />
{/key}
