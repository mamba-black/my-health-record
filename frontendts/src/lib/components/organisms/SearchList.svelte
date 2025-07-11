<script lang="ts">
  import { goto } from "$app/navigation";
  import type Patient from "$lib/domain/Patient";

  let { patients = $bindable() }: { patients: Patient[] } = $props();

  let gotoPatient = (e: Event, patient: Patient) => {
    e.preventDefault();
    goto(`/history/${patient.id}`, { state: patient });
  };
</script>

<ul role="list" class="divide-y divide-gray-100">
  {#each patients as patient}
    <li>
      <!--      onclick={patient_onclick(e, &patient1)}-->
      <a
        href="/history/{patient.id}"
        onclick={(e) => gotoPatient(e, patient)}
        class="flex justify-between gap-x-6 p-5 m-2 border rounded-lg border-transparent hover:border-blue-500 hover:bg-sky-100 hover:shadow-lg hover:cursor-pointer"
      >
        <!-- // href={private::HISTORY_DETAIL.replace(":id", &patient.id)}>-->
        <div class="flex gap-x-4">
          <!--                <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src={&patient.avatar.unwrap_or("".to_string())} alt="" />-->
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">
              {patient.firstName}
              <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                <span class="absolute inset-0 bg-green-200/50 rounded-full"></span>
                <span class="relative">Cita programada</span>
              </span>
            </p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{patient.email}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900"></p>
          <!--    if patient.online {Either::Left(view!{<>-->
          <div class="mt-1 flex items-center gap-x-1.5">
            <div class="flex-none rounded-full bg-emerald-500/20 p-1">
              <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
            </div>
            <p class="text-xs leading-5 text-gray-500">Online</p>
          </div>
          <!-- </>})} else {Either::Right(view!{<>-->
          <p class="mt-1 text-xs leading-5 text-gray-500">
            "Last seen "<time datetime="2023-01-23T13:23Z">3h ago</time>
          </p>
          <!-- </>})}-->
        </div>
      </a>
    </li>
  {/each}
</ul>
