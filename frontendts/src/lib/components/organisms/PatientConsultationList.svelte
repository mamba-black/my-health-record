<script lang="ts">
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import SubmitButton from "$lib/components/atoms/SubmitButton.svelte";
import Appointment from "$lib/domain/Appointment";
import Patient from "$lib/domain/Patient";
import "dayjs/locale/es";
import { goto } from "$app/navigation";
import STitle from "$lib/components/atoms/STitle.svelte";
import { log } from "$lib/services/LoggerService";
import { HISTORY } from "../../../routes/paths";

dayjs.extend(relativeTime);
dayjs.locale("es");

let {
  patient,
  appointments,
}: { patient: Patient; appointments: Appointment[] } = $props();

let now = dayjs();
let openPatientConsultation = $state(false);
let selectedAppointment: Appointment | undefined = $state();

let appointmentWrappers: { appointment: Appointment; date: string }[] =
  appointments.map((appointment) => {
    log.info("appointment", appointment);
    let date: string;
    if (now < appointment.date) {
      date = "dentro de " + appointment.date.from(now);
    } else {
      date = appointment.date.from(now);
    }
    return { appointment, date };
  });

function onClick(appointment: Appointment, e: Event) {
  log.info("appointment onClick", appointment);
  if (e instanceof KeyboardEvent && e.key !== "Enter") {
    return;
  }
  e.preventDefault();
  openPatientConsultation = !openPatientConsultation;
  selectedAppointment = appointment;
  goto(`${HISTORY}/${patient.id}/encounters/123/`, {
    state: { appointment: appointment, patient: patient },
  });
  log.debug("isOpen", openPatientConsultation);
}
</script>

<div class="">
  <STitle>Atenciones</STitle>
  <form>
    <div class="text-right">
      <SubmitButton label="Nueva consulta"/>
    </div>
    <ul class="divide-y divide-gray-100" role="list">
      {#each appointmentWrappers as appointmentWrapper}
        <li>
          <a
            role="cell" tabindex="0" href={`${HISTORY}/${patient.id}/encounters/123/`}
            class="flex justify-between gap-x-6 p-5 m-2 border rounded-lg border-transparent hover:border-blue-500 hover:bg-sky-100 hover:shadow-lg hover:cursor-pointer"
            onclick={e => onClick(appointmentWrapper.appointment, e)}
            onkeydown={e => onClick(appointmentWrapper.appointment, e)}
          >
            <div class="flex gap-x-4">
              <div class="flex-col justify-center items-center rounded-lg bg-white overflow-hidden shadow-md w-20">
                <div class="bg-blue-500 text-white py-1 px-1">
                  <p class="text-xs font-semibold text-white uppercase tracking-wide text-center">
                    {appointmentWrapper.appointment.date.format("MMMM")}
                    <span class="text-gray-300 leading-none">
                      <br>{appointmentWrapper.appointment.date.format("YYYY")}
                    </span>
                  </p>
                </div>
                <div class="flex-col justify-center items-center">
                  <p class="text-xs text-gray-400 text-center pt-1 px-4 leading-none">
                    {appointmentWrapper.appointment.date.format("dddd")}
                  </p>
                  <p class="font-bold text-black text-center pb-1.5 px-3 leading-none" style="font-size: 18px">
                    {appointmentWrapper.appointment.date.day()}
                  </p>
                </div>
              </div>

              <div class="min-w-0 flex-auto">
                <p class="text-sm font-semibold leading-6 text-gray-900">
                  Hector Miuler Malpcia Gallegos

                  <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                    <span class="absolute inset-0 bg-orange-300/50 rounded-full"></span>
                    {#if appointmentWrapper.appointment.date > now}
                      <span class="relative">Cita programada</span>
                    {:else}
                      <span></span>
                    {/if}
                  </span>
                  <span> </span>
                  {#if appointmentWrapper.appointment.exams.length > 0}
                    <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                      <span class="absolute inset-0 bg-green-200/50 rounded-full"></span>
                      <span class="relative"> Exemenes</span>
                    </span>
                  {:else}
                    <span> </span>
                  {/if}
                </p>
                <p class="mt-1 truncate text-xs leading-5 text-gray-500">
                  miuler@gmail.com
                </p>
              </div>
            </div>
            <div class="hidden sm:flex sm:flex-col sm:items-end">
              <!--            // <p class="text-sm leading-6 text-gray-900">{}</p>-->
              <!--            // {-->
              <!--            // if patient.online {view!{<>-->
              <!--            //     <div class="mt-1 flex items-center gap-x-1.5">-->
              <!--            //         <div class="flex-none rounded-full bg-emerald-500/20 p-1">-->
              <!--            //             <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>-->
              <!--            //         </div>-->
              <!--            //         <p class="text-xs leading-5 text-gray-500">Online</p>-->
              <!--            //     </div>-->
              <!--            // </>}} else {view!{<>-->
              <p class="mt-1 text-xs leading-5 text-gray-500">
                <time datetime={appointmentWrapper.appointment.date}>{appointmentWrapper.date}</time>
              </p>
              <!--            // </>}}-->
              <!--            // }-->
            </div>
          </a>
        </li>
      {/each}
    </ul>
  </form>
</div>
