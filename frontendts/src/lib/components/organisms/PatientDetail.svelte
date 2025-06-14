<script lang="ts">
  import Input from "$lib/components/atoms/Input.svelte";
  import ResetButton from "$lib/components/atoms/ResetButton.svelte";
  import SubmitButton from "$lib/components/atoms/SubmitButton.svelte";
  import Patient from "$lib/domain/Patient";
  import _ from "lodash";
  import MedicalConditions from "$lib/components/organisms/MedicalConditions.svelte";

  type PROPS = {
    patient: Patient,
    reset: boolean,
    setEditing?: (readOnly: boolean) => void,
  };

  let { patient, reset = $bindable(false), setEditing}: PROPS = $props();

  let patientCache = _.cloneDeep(patient);
  let _patient = $state(patient);
  let readOnly = $state(true);

  function reset_handle() {
    _patient = _.cloneDeep(patientCache);
    readOnly = true;
    setEditing?.(!readOnly);
  }

  function submit_handle(e: Event) {
    e.preventDefault();
    readOnly = !readOnly;
    setEditing?.(!readOnly);
    patientCache = _.cloneDeep(_patient);
  }

  $effect(() => {
    if (reset) {
      reset_handle();
      reset = false;
    }
  });
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
      <Input id="district" name="Distrito" bind:value={_patient.address.district} readonly={readOnly} />

      <!-- --------------------------------------------- -->
      <Input id="phone" name="Telefono" readonly={readOnly} bind:value={_patient.phone} _type="tel" />
      <Input id="email" name="Correo electronico" readonly={readOnly} bind:value={_patient.email} _type="email" />
      <Input id="birthday" name="Fecha de cumpleaños" readonly={readOnly} value={"birthday"} _type="date" />

      <!-- --------------------------------------------- -->
      <div class="col-span-3">
        <hr class="h-px mt-8" />
      </div>
      <MedicalConditions medical_conditions={_patient.medicalConditions} readonly={readOnly}></MedicalConditions>

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
