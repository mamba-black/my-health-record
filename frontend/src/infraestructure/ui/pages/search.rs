use yew::prelude::*;
use crate::infraestructure::ui::organisms::header::{MedicalHeader};

#[function_component]
pub fn Search() -> Html {
    html! {
    <div class="min-h-full">
      <MedicalHeader />

      <header class="bg-white shadow">
        <div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <h1 class="text-3xl font-bold tracking-tight text-gray-900">{"Busqueda de pacientes"}</h1>
        </div>
      </header>
      <main>
        <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
          <div class="px-10 py-10 bg-white rounded-2xl">
            <Grid />
          </div>
        </div>
      </main>
    </div>
    }
}

#[function_component]
fn Grid() -> Html {
    html! {
    <ul role="list" class="divide-y divide-gray-100">
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">
              {"Leslie Alexander "}
              <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                <span  class="absolute inset-0 bg-green-200 opacity-50 rounded-full"></span>
                <span class="relative">{"Cita pendiente"}</span>
              </span>
            </p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"leslie.alexander@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Co-Founder / CEO"}</p>
          <p class="mt-1 text-xs leading-5 text-gray-500">{"Last seen "}<time datetime="2023-01-23T13:23Z">{"3h ago"}</time></p>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1519244703995-f4e0f30006d5?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Michael Foster"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"michael.foster@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Co-Founder / CTO"}</p>
          <p class="mt-1 text-xs leading-5 text-gray-500">{"Last seen "}<time datetime="2023-01-23T13:23Z">{"3h ago"}</time></p>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1506794778202-cad84cf45f1d?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Dries Vincent"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"dries.vincent@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Business Relations"}</p>
          <div class="mt-1 flex items-center gap-x-1.5">
            <div class="flex-none rounded-full bg-emerald-500/20 p-1">
              <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
            </div>
            <p class="text-xs leading-5 text-gray-500">{"Online"}</p>
          </div>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1517841905240-472988babdf9?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Lindsay Walton"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"lindsay.walton@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Front-end Developer"}</p>
          <p class="mt-1 text-xs leading-5 text-gray-500">{"Last seen "}<time datetime="2023-01-23T13:23Z">{"3h ago"}</time></p>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Courtney Henry"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"courtney.henry@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Designer"}</p>
          <p class="mt-1 text-xs leading-5 text-gray-500">{"Last seen "}<time datetime="2023-01-23T13:23Z">{"3h ago"}</time></p>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Tom Cook"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"tom.cook@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Director of Product"}</p>
          <div class="mt-1 flex items-center gap-x-1.5">
            <div class="flex-none rounded-full bg-emerald-500/20 p-1">
              <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
            </div>
            <p class="text-xs leading-5 text-gray-500">{"Online"}</p>
          </div>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Tom Cook"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"tom.cook@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Director of Product"}</p>
          <div class="mt-1 flex items-center gap-x-1.5">
            <div class="flex-none rounded-full bg-emerald-500/20 p-1">
              <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
            </div>
            <p class="text-xs leading-5 text-gray-500">{"Online"}</p>
          </div>
        </div>
      </li>
      <li class="flex justify-between gap-x-6 py-5">
        <div class="flex gap-x-4">
          <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" />
          <div class="min-w-0 flex-auto">
            <p class="text-sm font-semibold leading-6 text-gray-900">{"Tom Cook"}</p>
            <p class="mt-1 truncate text-xs leading-5 text-gray-500">{"tom.cook@example.com"}</p>
          </div>
        </div>
        <div class="hidden sm:flex sm:flex-col sm:items-end">
          <p class="text-sm leading-6 text-gray-900">{"Director of Product"}</p>
          <div class="mt-1 flex items-center gap-x-1.5">
            <div class="flex-none rounded-full bg-emerald-500/20 p-1">
              <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
            </div>
            <p class="text-xs leading-5 text-gray-500">{"Online"}</p>
          </div>
        </div>
      </li>
    </ul>
    }
}