<script lang="ts">
  import Input from "$lib/components/atoms/Input.svelte";
  import ResetButton from "$lib/components/atoms/ResetButton.svelte";
  import SubmitButton from "$lib/components/atoms/SubmitButton.svelte";
  import Patient from "$lib/Patient";

  let { patient }: { patient: Patient} = $props();
  let patientCache = { ...patient };

  let _patient = $state(patient);
  let read_only = $state(true);


  function reset_handle() {
    console.debug("patient original:", patientCache);
    console.debug("patient modificado:", $state.snapshot(_patient));
    _patient = { ...patientCache };
    read_only = true;
  }

  function submit_handle() {
    console.debug("patient original:", patientCache);
    console.debug("patient modificado:", $state.snapshot(_patient));
    read_only = !read_only;
    patientCache = { ..._patient };
  }


</script>

<div class="lg:wa-7/12 lg:justify-around">
  <form onsubmit={submit_handle}>
    <div class="md:grid md:grid-cols-3 md:gap-4 space-y-5 md:space-y-0">
      <!-- --------------------------------------------- -->
      <Input id="firstName" name="Nombre" bind:value={_patient.firstName} readonly={read_only} required={true} />
      <Input id="lastName" name="Apellido Paterno" bind:value={_patient.lastName} readonly={read_only} required={true} />
      <Input id="secondLastName" name="Apellido Materno" bind:value={_patient.secondLastName} readonly={read_only} />

      <!-- --------------------------------------------- -->
      <Input id="street" name="Dirección" value={_patient.address.street} _class="col-span-2" readonly={read_only} />
      <Input id="district" name="Distrito" value={"district"} readonly={read_only} />

      <!-- --------------------------------------------- -->
      <Input id="phone" name="Telefono" readonly={read_only} value={"phone"} _type="tel" />
      <Input id="email" name="Correo electronico" readonly={read_only} value={"email"} _type="email" />
      <Input id="birthday" name="Fecha de cumpleaños" readonly={read_only} value={"birthday"} _type="date" />

      <!-- --------------------------------------------- -->
      <div class="col-span-3">
        <hr class="h-px mt-8" />
      </div>
      <h2 class="col-span-3">Antecedentes</h2>

      <!-- --------------------------------------------- -->
      <Input id="alergy" name="Alergias" readonly={read_only} value={"allergies"} _class="col-span-3" />

      <!-- --------------------------------------------- -->
      <!--      <Checkbox name="Hepatitis" value={hepatitis} set_value={set_hepatitis} readonly={read_only} />-->
      <!--      <Checkbox name="Diabetes" value={diabetes} set_value={set_diabetes} readonly={read_only} />-->
      <!--      <Checkbox name="Hemorragia" value={hemorrhage} set_value={set_hemorrhage} readonly={read_only} />-->
      <!--      <Checkbox name="Presion alta" value={high_pressure} set_value={set_high_pressure} readonly={read_only} />-->
      <!--      <Checkbox name="Presion baja" value={low_pressure} set_value={set_low_pressure} readonly={read_only} />-->
      <!--      <Checkbox name="Colesterol" value={cholesterol} set_value={set_cholesterol} readonly={read_only} />-->
      <!--      <Checkbox name="Asma" value={asthma} set_value={set_asthma} readonly={read_only} />-->
      <!--      <Checkbox name="TBC" value={tbc} set_value={set_tbc} readonly={read_only} />-->

      <!-- --------------------------------------------- -->
      <!--      //-->

      <!-- --------------------------------------------- -->
      <!--      //-->

      <!-- --------------------------------------------- -->
      <div class="col-span-2"></div>
      <div class="justify-self-end">
        <!--{edit_button}-->
        {#if read_only}
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
