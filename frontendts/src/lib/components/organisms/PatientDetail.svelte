<script lang="ts">
  import Input from "$lib/components/atoms/Input.svelte";
  import ResetButton from "$lib/components/atoms/ResetButton.svelte";
  import SubmitButton from "$lib/components/atoms/SubmitButton.svelte";
  import Patient from "$lib/domain/Patient";
  import MedicalConditionsComponent from "$lib/components/organisms/MedicalConditions.svelte";
  import STitle from "$lib/components/atoms/STitle.svelte";
  import {goto} from "$app/navigation";
  import {log} from "$lib/services/LoggerService";
  import type MedicalConditions from "$lib/domain/MedicalConditions";
  import {base} from "$app/paths";

  type PROPS = {
    patient: Patient,
    reset: boolean,
    setEditing?: (readOnly: boolean) => void,
  };

  const toPlainObject = (p: any) => JSON.parse(JSON.stringify(p));

  let {patient, reset = $bindable(false), setEditing}: PROPS = $props();

  // let patientCache = _.cloneDeep(patient);
  let patientCache: Patient = toPlainObject(patient);
  // let _patient_clone = _.cloneDeep(patient);
  let _patient = $state<Patient>(toPlainObject(patientCache));
  let _medicalConditions = $state<MedicalConditions>(toPlainObject(patientCache.medicalConditions));
  let readOnly = $state(true);

  function reset_handle() {
    log.debug("reset/patientCache:", patientCache);
    log.debug("reset/_patient:", $state.snapshot(_patient));
    log.debug("reset/_medicalConditions:", $state.snapshot(_medicalConditions));
    // _patient = _.cloneDeep(patientCache);
    _patient = toPlainObject(patientCache);
    _medicalConditions = toPlainObject(patientCache.medicalConditions)
    readOnly = true;
    setEditing?.(!readOnly);
  }

  function submit_handle(e: Event) {
    e.preventDefault();
    readOnly = !readOnly;
    setEditing?.(!readOnly);
    // patientCache = _.cloneDeep(_patient);
    patientCache = toPlainObject(_patient);
    log.debug("submit/_patient", $state.snapshot(_patient));
    log.debug("submit/patientCache", patientCache);
    log.debug("submit/_medicalConditions:", $state.snapshot(_medicalConditions));

    if (readOnly) {
      // const serializablePatient = JSON.parse(JSON.stringify(_patient));
      _patient.medicalConditions = $state.snapshot(_medicalConditions);
      log.info("Guardar el estado", $state.snapshot(_patient));
      patientCache = toPlainObject(_patient);
      goto(`${base}/history/${_patient.id}`, { state: patientCache, replaceState: true });
    }
  }

  $effect(() => {
    if (reset) {
      reset_handle();
      reset = false;
    }
  });
</script>

<form onsubmit={submit_handle}>
  <div class="md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">
    <STitle>Datos del Paciente</STitle>
    <!-- --------------------------------------------- -->
    <Input id="firstName" name="Nombre" bind:value={_patient.firstName} readonly={readOnly} required={true}/>
    <Input id="lastName" name="Apellido Paterno" bind:value={_patient.lastName} readonly={readOnly} required={true}/>
    <Input id="secondLastName" name="Apellido Materno" bind:value={_patient.secondLastName} readonly={readOnly}/>

    <!-- --------------------------------------------- -->
    <Input
      id="street"
      name="Dirección"
      bind:value={_patient.address.street}
      _class="col-span-2"
      readonly={readOnly}
    />
    <Input id="district" name="Distrito" bind:value={_patient.address.district} readonly={readOnly}/>

    <!-- --------------------------------------------- -->
    <Input id="phone" name="Telefono" readonly={readOnly} bind:value={_patient.phone} _type="tel"/>
    <Input id="email" name="Correo electronico" readonly={readOnly} bind:value={_patient.email} _type="email"/>
    <Input id="birthday" name="Fecha de cumpleaños" readonly={readOnly} value={"birthday"} _type="date"/>

    <!-- --------------------------------------------- -->
    <div />
    <div class="col-span-3">
    <MedicalConditionsComponent bind:medical_conditions={_medicalConditions} readonly={readOnly}></MedicalConditionsComponent>
    </div>

    <!-- --------------------------------------------- -->
    <div class="col-span-2"></div>
    <div class="justify-self-end">
      <!--{edit_button}-->
      {#if readOnly}
        <div>
          <SubmitButton label="Editar"/>
        </div>
      {:else}
        <div class="space-x-3">
          <SubmitButton label="Guardar"/>
          <ResetButton label="Cancelar" onclick={reset_handle}/>
        </div>
      {/if}
    </div>
  </div>
</form>
