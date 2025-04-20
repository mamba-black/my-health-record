<script>
  import Checkbox from "$lib/components/atoms/Checkbox.svelte";
  import Input from "$lib/components/atoms/Input.svelte";
  import MedicalConditions from "$lib/domain/MedicalConditions";
  // import _ from "lodash";

  let medical_conditions = $props();
  let _medical_conditions = $state(medical_conditions);
  let readOnly = $state(true);
</script>

<div class="col-span-3 md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">
  <h3 class="col-span-3 text-2xl font-bold tracking-tight text-gray-900 pt-1">Antecedentes</h3>

  <!-- --------------------------------------------- -->
  <Input id="allergy" name="Alergias" readonly={readOnly} value={"allergies"} _class="col-span-3" />

  <!-- --------------------------------------------- -->
  <Checkbox id="hepatitis" name="Hepatitis" bind:value={_medical_conditions.hepatitis} readonly={readOnly} />
  <Checkbox id="diabetes" name="Diabetes" bind:value={_medical_conditions.diabetes} readonly={readOnly} />
  <Checkbox id="hemorrhage" name="Hemorragia" bind:value={_medical_conditions.hemorrhage} readonly={readOnly} />
  <Checkbox
    id="highPressure"
    name="Presion alta"
    bind:value={_medical_conditions.highPressure}
    readonly={readOnly}
  />
  <Checkbox id="lowPressure" name="Presion baja" bind:value={_medical_conditions.lowPressure} readonly={readOnly} />
  <Checkbox id="cholesterol" name="Colesterol" bind:value={_medical_conditions.cholesterol} readonly={readOnly} />
  <Checkbox id="asthma" name="Asma" bind:value={_medical_conditions.asthma} readonly={readOnly} />
  <Checkbox id="tbc" name="TBC" bind:value={_medical_conditions.tbc} readonly={readOnly} />
</div>
