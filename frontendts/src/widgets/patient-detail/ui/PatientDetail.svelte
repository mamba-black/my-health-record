<script lang="ts">
import { goto } from "$app/navigation";
import {
  type MedicalConditions,
  MedicalConditionsCard,
  type Patient,
} from "$entities/patient";
import { HISTORY } from "$shared/config/routes";
import { log } from "$shared/lib/logger";
import { Input, ResetButton, STitle, SubmitButton } from "$shared/ui";

type Props = {
  patient: Patient;
  reset: boolean;
  setEditing?: (readOnly: boolean) => void;
};

const toPlainObject = <T,>(p: T): T => JSON.parse(JSON.stringify(p));

let { patient, reset = $bindable(false), setEditing }: Props = $props();

let patientCache: Patient = toPlainObject(patient);
let _patient = $state<Patient>(toPlainObject(patientCache));
let _medicalConditions = $state<MedicalConditions>(
  toPlainObject(patientCache.medicalConditions),
);
let readOnly = $state(true);

function reset_handle() {
  log.debug("reset/patientCache:", patientCache);
  log.debug("reset/_patient:", $state.snapshot(_patient));
  log.debug("reset/_medicalConditions:", $state.snapshot(_medicalConditions));
  _patient = toPlainObject(patientCache);
  _medicalConditions = toPlainObject(patientCache.medicalConditions);
  readOnly = true;
  setEditing?.(!readOnly);
}

function submit_handle(e: Event) {
  e.preventDefault();
  readOnly = !readOnly;
  setEditing?.(!readOnly);
  patientCache = toPlainObject(_patient);
  log.debug("submit/_patient", $state.snapshot(_patient));
  log.debug("submit/patientCache", patientCache);
  log.debug("submit/_medicalConditions:", $state.snapshot(_medicalConditions));

  if (readOnly) {
    _patient.medicalConditions = $state.snapshot(_medicalConditions);
    log.info("Guardar el estado", $state.snapshot(_patient));
    patientCache = toPlainObject(_patient);
    goto(`${HISTORY}/${_patient.id}`, {
      state: patientCache,
      replaceState: true,
    });
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
    <Input bind:value={_patient.firstName} id="firstName" name="Nombre" readonly={readOnly} required={true}/>
    <Input bind:value={_patient.lastName} id="lastName" name="Apellido Paterno" readonly={readOnly} required={true}/>
    <Input bind:value={_patient.secondLastName} id="secondLastName" name="Apellido Materno" readonly={readOnly}/>

    <!-- --------------------------------------------- -->
    <Input
      _class="col-span-2"
      bind:value={_patient.address.street}
      id="street"
      name="Dirección"
      readonly={readOnly}
    />
    <Input bind:value={_patient.address.district} id="district" name="Distrito" readonly={readOnly}/>

    <!-- --------------------------------------------- -->
    <Input _type="tel" bind:value={_patient.phone} id="phone" name="Telefono" readonly={readOnly}/>
    <Input _type="email" bind:value={_patient.email} id="email" name="Correo electronico" readonly={readOnly}/>
    <Input _type="date" id="birthday" name="Fecha de cumpleaños" readonly={readOnly} value={"birthday"}/>

    <!-- --------------------------------------------- -->
    <div></div>
    <div class="col-span-3">
      <MedicalConditionsCard
        bind:medical_conditions={_medicalConditions}
        readonly={readOnly}
      />
    </div>

    <!-- --------------------------------------------- -->
    <div class="col-span-2"></div>
    <div class="justify-self-end">
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
