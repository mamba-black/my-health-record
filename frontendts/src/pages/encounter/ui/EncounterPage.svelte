<script lang="ts">
import { page } from "$app/state";
import type { EncounterEntity } from "$entities/encounter";
import type { Patient } from "$entities/patient";
import { log } from "$shared/lib/logger";
import { EncounterDetail } from "$widgets/encounter-detail";

let {
  encounterEntity,
  patient,
}: { encounterEntity: EncounterEntity; patient: Patient } = page.state as {
  encounterEntity: EncounterEntity;
  patient: Patient;
};

log.info("encounterEntity", encounterEntity);
log.info("encounterEntity.date", encounterEntity.date);
</script>

<div>
  <EncounterDetail
    bind:encounterEntity={encounterEntity}
    isOpen={true}
    patient={patient}
  />
</div>
