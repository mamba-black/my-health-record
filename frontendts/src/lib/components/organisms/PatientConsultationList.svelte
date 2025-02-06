<script lang="ts">
  import Patient from "$lib/Patient";
  import SubmitButton from "$lib/components/atoms/SubmitButton.svelte";
  import Appointment from "$lib/Appointment";

  let { patient }: { patient: Patient } = $props();

  let appointments: any[] = [1, 2];
  let now = new Date();

  function onClick(appointment: Appointment, e: Event) {
    e.preventDefault();
    console.debug("onClick")
  }
</script>

<div class="lg:wa-7/12 lg:justify-around">
  <form>
    <div class="text-right"><SubmitButton label="Nueva consulta" /></div>
    <ul role="list" class="divide-y divide-gray-100">
      {#each appointments as appointment}
<!--      <For-->
<!--        children= move |appointment: Appointment| {-->
<!--            let month = Month::try_from(u8::try_from(appointment.date.month()).unwrap()).unwrap();-->
<!--            let mut formatter = Formatter::with_language(Spanish);-->
<!--            let past = if (now > appointment.date) {-->
<!--              formatter.convert_chrono(appointment.date, now)-->
<!--            } else {-->
<!--              formatter.ago("dentro de ").convert_chrono(now, appointment.date)-->
<!--            };-->
<!--            let appointment2 = appointment.clone();-->

        <li>
          <a class="flex justify-between gap-x-6 p-5 m-2 border rounded-lg border-transparent hover:border-blue-500 hover:bg-sky-100 hover:shadow-lg hover:cursor-pointer" href="" onclick={e => onClick(appointment, e)}>
            <div class="flex gap-x-4">

            <div class="flex-col justify-center items-center rounded-lg bg-white overflow-hidden shadow-md w-20">
              <div class="bg-blue-500 text-white py-1 px-1">
                <p class="text-xs font-semibold text-white uppercase tracking-wide text-center">
                  <!--{appointment.date.format_localized("%B", Locale::es_ES)}-->
                  Enero
                  <span class="text-gray-300 leading-none">
                    <!--{appointment.date.format_localized("%A", Locale::es_ES)}-->
                    <br>2025
                  </span>
                </p>
              </div>
              <div class="flex-col justify-center items-center">
                <p class="text-xs text-gray-400 text-center pt-1 px-4 leading-none">
                  <!--{appointment.date.format_localized("%A", Locale::es_ES)}-->
                  Martes
                </p>
                <p class="font-bold text-black text-center pb-1.5 px-3 leading-none" style="font-size: 18px;">
                  <!--{appointment.date.day()}-->
                  3
                </p>
              </div>
            </div>


              <div class="min-w-0 flex-auto">
                <p class="text-sm font-semibold leading-6 text-gray-900">
                  Hector Miuler Malpcia Gallegos

                  <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                    <span  class="absolute inset-0 bg-orange-300 opacity-50 rounded-full"></span>
                  <!--{-->
                  <!--if appointment.date > now {-->
                  <!--Either::Left(view!{<span class="relative">Cita programada</span>})-->
                  <!--} else {-->
                  <!--Either::Right(view!{<span></span>})-->
                  <!--}-->
                  <!--}-->
                  </span>
                  <span>  </span>
                  <!--{-->
                  <!--if appointment.exams.is_some() {-->
                  <!--Either::Left(view! {-->
                  <!--<span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">-->
                  <!--<span class="absolute inset-0 bg-green-200 opacity-50 rounded-full"></span>-->
                  <!--<span class="relative"> Exemenes</span>-->
                  <!--</span>-->
                  <!--})-->
                  <!--} else {-->
                  <!--Either::Right(view!{<span></span>})-->
                  <!--}-->
                  <!--}-->
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
<!--            <p class="mt-1 text-xs leading-5 text-gray-500"><time datetime={appointment.date}>{past}</time></p>-->
<!--            // </>}}-->
<!--            // }-->
            </div>
          </a>
        </li>
      {/each}
    </ul>
  </form>
</div>
