<script lang="ts">
  import Checkbox from "$lib/components/atoms/Checkbox.svelte";
  import Input from "$lib/components/atoms/Input.svelte";
  import MedicalConditionsClass from "$lib/domain/MedicalConditions";
  import {log} from "$lib/services/LoggerService";
  import STitle from "$lib/components/atoms/STitle.svelte";
  // import _ from "lodash";

  let {readonly, medical_conditions = $bindable()}: {
    readonly: Boolean,
    medical_conditions: MedicalConditionsClass
  } = $props();
  let _medical_conditions = $state(medical_conditions);
  log.debug("readonly", readonly);
  log.debug("medical_conditions", medical_conditions);
  log.debug("medical_conditions", _medical_conditions);
</script>

<div class="mb-8">
  <STitle>Antecedentes</STitle>
  <div
    class="bg-blue-50 border border-blue-200 text-blue-800 p-4 rounded-md col-span-3 md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">

    <!-- --------------------------------------------- -->
    <Input id="allergy" name="Alergias" readonly={readonly} value={"allergies"} _class="col-span-3" />

    <!-- --------------------------------------------- -->
    <Checkbox id="hepatitis" name="Hepatitis" bind:value={_medical_conditions.hepatitis} readonly={readonly}/>
    <Checkbox id="diabetes" name="Diabetes" bind:value={_medical_conditions.diabetes} readonly={readonly}/>
    <Checkbox id="hemorrhage" name="Hemorragia" bind:value={_medical_conditions.hemorrhage} readonly={readonly}/>
    <Checkbox
      id="highPressure"
      name="Presion alta"
      bind:value={_medical_conditions.highPressure}
      readonly={readonly}
    />
    <Checkbox id="lowPressure" name="Presion baja" bind:value={_medical_conditions.lowPressure} readonly={readonly}/>
    <Checkbox id="cholesterol" name="Colesterol" bind:value={_medical_conditions.cholesterol} readonly={readonly}/>
    <Checkbox id="asthma" name="Asma" bind:value={_medical_conditions.asthma} readonly={readonly}/>
    <Checkbox id="tbc" name="TBC" bind:value={_medical_conditions.tbc} readonly={readonly}/>
  </div>
</div>
