<script lang="ts">
  import Checkbox from "$lib/components/atoms/Checkbox.svelte";
  import Input from "$lib/components/atoms/Input.svelte";
  import ResetButton from "$lib/components/atoms/ResetButton.svelte";
  import SubmitButton from "$lib/components/atoms/SubmitButton.svelte";
  import Patient from "$lib/domain/Patient";
  import _ from "lodash";

  let { patient }: { patient: Patient } = $props();
  let patientCache = _.cloneDeep(patient);
  let _patient = $state(patient);
  let readOnly = $state(true);

  function reset_handle() {
    console.debug("patient original:", patientCache);
    console.debug("patient modificado:", $state.snapshot(_patient));
    _patient = _.cloneDeep(patientCache);
    readOnly = true;
  }

  function submit_handle() {
    console.debug("patient original:", patientCache);
    console.debug("patient modificado:", $state.snapshot(_patient));
    readOnly = !readOnly;
    patientCache = _.cloneDeep(_patient);
  }
</script>

<div class="lg:wa-7/12 lg:justify-around">
  <form onsubmit={submit_handle}>
    <div class="md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">
      <!-- --------------------------------------------- -->
      <Input id="firstName" name="Nombre" bind:value={_patient.firstName} readonly={readOnly} required={true} />
      <Input id="lastName" name="Apellido Paterno" bind:value={_patient.lastName} readonly={readOnly} required={true} />
      <Input id="secondLastName" name="Apellido Materno" bind:value={_patient.secondLastName} readonly={readOnly} />

      <!-- --------------------------------------------- -->
      <Input
        id="street"
        name="Dirección"
        bind:value={_patient.address.street}
        _class="col-span-2"
        readonly={readOnly}
      />
      <Input id="district" name="Distrito" value={_patient.address.district} readonly={readOnly} />

      <!-- --------------------------------------------- -->
      <Input id="phone" name="Telefono" readonly={readOnly} value={_patient.phone} _type="tel" />
      <Input id="email" name="Correo electronico" readonly={readOnly} value={_patient.email} _type="email" />
      <Input id="birthday" name="Fecha de cumpleaños" readonly={readOnly} value={"birthday"} _type="date" />

      <!-- --------------------------------------------- -->
      <div class="col-span-3">
        <hr class="h-px mt-8" />
      </div>
      <h2 class="col-span-3">Antecedentes</h2>

      <!-- --------------------------------------------- -->
      <Input id="allergy" name="Alergias" readonly={readOnly} value={"allergies"} _class="col-span-3" />

      <!-- --------------------------------------------- -->
      <Checkbox id="hepatitis" name="Hepatitis" bind:value={_patient.allergies.hepatitis} readonly={readOnly} />
      <Checkbox id="diabetes" name="Diabetes" bind:value={_patient.allergies.diabetes} readonly={readOnly} />
      <Checkbox id="hemorrhage" name="Hemorragia" bind:value={_patient.allergies.hemorrhage} readonly={readOnly} />
      <Checkbox
        id="highPressure"
        name="Presion alta"
        bind:value={_patient.allergies.highPressure}
        readonly={readOnly}
      />
      <Checkbox id="lowPressure" name="Presion baja" bind:value={_patient.allergies.lowPressure} readonly={readOnly} />
      <Checkbox id="cholesterol" name="Colesterol" bind:value={_patient.allergies.cholesterol} readonly={readOnly} />
      <Checkbox id="asthma" name="Asma" bind:value={_patient.allergies.asthma} readonly={readOnly} />
      <Checkbox id="tbc" name="TBC" bind:value={_patient.allergies.tbc} readonly={readOnly} />

      <!-- --------------------------------------------- -->
      <div class="col-span-2"></div>
      <div class="justify-self-end">
        <!--{edit_button}-->
        {#if readOnly}
          <div><SubmitButton label="Editar" /></div>
        {:else}
          <div class="space-x-3">
            <SubmitButton label="Guardar" />
            <ResetButton label="Cancelar" onclick={reset_handle} />
          </div>
        {/if}
      </div>
    </div>
  </form>
</div>
