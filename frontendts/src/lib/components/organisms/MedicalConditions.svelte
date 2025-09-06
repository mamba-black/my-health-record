<script lang="ts">
import Checkbox from "$lib/components/atoms/Checkbox.svelte";
import Input from "$lib/components/atoms/Input.svelte";
import STitle from "$lib/components/atoms/STitle.svelte";
import MedicalConditionsClass from "$lib/domain/MedicalConditions";
import { log } from "$lib/services/LoggerService";

// import _ from "lodash";

let {
  readonly,
  medical_conditions = $bindable(),
}: {
  readonly: Boolean;
  medical_conditions: MedicalConditionsClass;
} = $props();
// let _medical_conditions = $state(medical_conditions);
log.debug("readonly", readonly);
log.debug("medical_conditions", medical_conditions);
// log.debug("_medical_conditions", _medical_conditions);
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
