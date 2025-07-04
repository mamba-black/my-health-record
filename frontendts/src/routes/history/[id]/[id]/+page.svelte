<script lang="ts">
  import {log} from "$lib/services/LoggerService";
  import Appointment from "$lib/domain/Appointment";
  import {page} from "$app/state";


  let appointment: Appointment = page.state as Appointment;

  log.info("appointment", appointment);
  log.info("appointment.date", appointment.date);
</script>

<div>
[{appointment.date}]
  Hola
</div>
