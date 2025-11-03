<script lang="ts">
import { page } from "$app/state";
import PatientConsultation from "$lib/components/organisms/PatientConsultation.svelte";
import Appointment from "$lib/domain/Appointment";
import type Patient from "$lib/domain/Patient";
import { log } from "$lib/services/LoggerService";

let { appointment, patient }: { appointment: Appointment; patient: Patient } =
  page.state as {
    appointment: Appointment;
    patient: Patient;
  };

log.info("appointment", appointment);
log.info("appointment.date", appointment.date);
</script>

<div>
  <PatientConsultation bind:appointment={appointment}
                       isOpen={true}
                       patient={patient}/>
</div>
