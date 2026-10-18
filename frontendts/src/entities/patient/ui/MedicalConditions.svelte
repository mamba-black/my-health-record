<script lang="ts">
import { log } from "$shared/lib/logger";
import { Checkbox, Input, STitle } from "$shared/ui";
import type MedicalConditionsClass from "../model/MedicalConditions";

let {
  readonly,
  medical_conditions = $bindable(),
}: {
  readonly: boolean;
  medical_conditions: MedicalConditionsClass;
} = $props();

log.debug("readonly", readonly);
log.debug("medical_conditions", medical_conditions);
</script>

<div class="mb-8">
  <STitle>Antecedentes</STitle>
  <div
    class="bg-blue-50 border border-blue-200 text-blue-800 p-4 rounded-md col-span-3 md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">

    <!-- --------------------------------------------- -->
    <Input _class="col-span-3" id="allergy" name="Alergias" readonly={readonly} value={"allergies"}/>

    <!-- --------------------------------------------- -->
    <Checkbox bind:value={medical_conditions.hepatitis} id="hepatitis" name="Hepatitis" readonly={readonly}/>
    <Checkbox bind:value={medical_conditions.diabetes} id="diabetes" name="Diabetes" readonly={readonly}/>
    <Checkbox bind:value={medical_conditions.hemorrhage} id="hemorrhage" name="Hemorragia" readonly={readonly}/>
    <Checkbox
      bind:value={medical_conditions.highPressure}
      id="highPressure"
      name="Presion alta"
      readonly={readonly}
    />
    <Checkbox bind:value={medical_conditions.lowPressure} id="lowPressure" name="Presion baja" readonly={readonly}/>
    <Checkbox bind:value={medical_conditions.cholesterol} id="cholesterol" name="Colesterol" readonly={readonly}/>
    <Checkbox bind:value={medical_conditions.asthma} id="asthma" name="Asma" readonly={readonly}/>
    <Checkbox bind:value={medical_conditions.tbc} id="tbc" name="TBC" readonly={readonly}/>
  </div>
</div>
