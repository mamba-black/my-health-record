<script lang="ts">
import { page } from "$app/state";
import Encounter from "$lib/components/organisms/Encounter.svelte";
import EncounterEntity from "$lib/domain/EncounterEntity";
import type Patient from "$lib/domain/Patient";
import { log } from "$lib/services/LoggerService";

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
  <Encounter bind:encounterEntity={encounterEntity}
             isOpen={true}
             patient={patient}/>
</div>
