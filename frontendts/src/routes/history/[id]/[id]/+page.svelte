<script lang="ts">
  import {log} from "$lib/services/LoggerService";
  import Appointment from "$lib/domain/Appointment";
  import {page} from "$app/state";
  import PatientConsultation from "$lib/components/organisms/PatientConsultation.svelte";
  import type Patient from "$lib/domain/Patient";


  let {appointment, patient}: {appointment: Appointment, patient: Patient} = page.state as {appointment: Appointment, patient: Patient};

  log.info("appointment", appointment);
  log.info("appointment.date", appointment.date);
</script>

<div>
  <PatientConsultation isOpen={true}
                       bind:appointment={appointment}
                       patient={patient} />
</div>
