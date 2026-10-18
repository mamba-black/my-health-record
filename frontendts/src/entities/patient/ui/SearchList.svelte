<script lang="ts">
import { goto } from "$app/navigation";
import { HISTORY } from "$shared/config/routes";
import type Patient from "../model/Patient";

let { patients = $bindable() }: { patients: Patient[] } = $props();

const gotoPatient = (e: Event, patient: Patient) => {
  e.preventDefault();
  goto(`${HISTORY}/${patient.id}`, { state: patient });
};
</script>

<ul class="divide-y divide-gray-100" role="list">
  {#each patients as patient}
    <li>
      <a
        href="{HISTORY}/{patient.id}"
        onclick={(e) => gotoPatient(e, patient)}
        class="flex justify-between gap-x-6 p-5 m-2 border rounded-lg border-transparent hover:border-blue-500 hover:bg-sky-100 hover:shadow-lg hover:cursor-pointer"
      >
        <div class="flex gap-x-4">
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">
              {patient.firstName} {patient.lastName} {patient.secondLastName ?? ""}
              <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                <span class="absolute inset-0 bg-green-200/50 rounded-full"></span>
                <span class="relative">Cita programada</span>
              </span>
            </p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{patient.email ?? ""}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900"></p>
          <div class="mt-1 flex items-center gap-x-1.5">
            <div class="flex-none rounded-full bg-emerald-500/20 p-1">
              <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
            </div>
            <p class="text-xs leading-5 text-gray-500">Online</p>
          </div>
          <p class="mt-1 text-xs leading-5 text-gray-500">
            "Last seen "
            <time datetime="2023-01-23T13:23Z">3h ago</time>
          </p>
        </div>
      </a>
    </li>
  {/each}
</ul>
