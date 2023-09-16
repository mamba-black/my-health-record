use leptos::*;
use leptos_router::*;

use crate::components::route::private;
use crate::components::route::public;

const MENU_HIDDEN_CSS: &str = "transition ease-in duration-75 transform opacity-0 scale-95       absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none";
const MENU_SHOW_CSS: &str = "transition ease-out duration-100 transform opacity-100 scale-100    absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none";

#[component]
pub fn MedicalHeader() -> impl IntoView {
    // let menu_css = use_state(Vec::<&str>::new);
    // let toggle_menu_status = use_state(|| false);

    // let menu_css = if *toggle_menu_status {
    let menu_css = if false {
        MENU_SHOW_CSS
    } else {
        MENU_HIDDEN_CSS
    };

    // let toggle_menu = Callback::from(move |_: MouseEvent| {
    //     log!("prueba de toggle_menu");
    //     // let a: EventTarget = e.target().unwrap();
    //     // let e: HtmlElement = a.unchecked_into::<HtmlElement>();
    //     // let js: JsValue = e.clone().into();
    //     // // info!("prueba de toggle_menu: {:?}", js);
    //     // use web_sys::console;
    //     // console::log_2(&"Hello using web-sys".into(), &js);
    //
    //     toggle_menu_status.set(!*toggle_menu_status);
    // });

    view! {
    <nav class="bg-gray-800">
      <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div class="flex h-16 items-center justify-between">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <img class="h-8 w-8" src="https://cdn-icons-png.flaticon.com/512/387/387552.png" alt="Your Company" />
            </div>
            {header_desktop()}
          </div>
          <div class="hidden md:block">
            <div class="ml-4 flex items-center md:ml-6">
              <button type="button" class="rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800">
                <span class="sr-only">View notifications</span>
                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0" />
                </svg>
              </button>

              //<!-- Profile dropdown -->
              <div class="relative ml-3">
                <div>
                  <button class="flex max-w-xs items-center rounded-full bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
                          // type="button" id="user-menu-button" aria-expanded="false" aria-haspopup="true" onclick={toggle_menu}>
                          type="button" id="user-menu-button" aria-expanded="false" aria-haspopup="true">
                    <span class="sr-only">Open user menu</span>
                    <img class="h-8 w-8 rounded-full" src="/img/doctor.webp" alt=""/>
                  </button>
                </div>

                //<!--
                //  Dropdown menu, show/hide based on menu state.
                //
                //  Entering: "transition ease-out duration-100"
                //    From: "transform opacity-0 scale-95"
                //    To: "transform opacity-100 scale-100"
                //  Leaving: "transition ease-in duration-75"
                //    From: "transform opacity-100 scale-100"
                //    To: "transform opacity-0 scale-95"
                //-->
                <div class={menu_css} role="menu" aria-orientation="vertical" aria-labelledby="user-menu-button" tabindex="-1">
                  //<!-- Active: "bg-gray-100", Not Active: "" -->
                  <a href="#" class="block px-4 py-2 text-sm text-gray-700" role="menuitem" tabindex="-1" id="user-menu-item-0">Your Profile</a>
                  <a href="#" class="block px-4 py-2 text-sm text-gray-700" role="menuitem" tabindex="-1" id="user-menu-item-1">Settings</a>
                  <a href="#" class="block px-4 py-2 text-sm text-gray-700" role="menuitem" tabindex="-1" id="user-menu-item-2">Sign out</a>
                </div>
              </div>
            </div>
          </div>
          <div class="-mr-2 flex md:hidden">
            //<!-- Mobile menu button -->
            <button type="button" class="inline-flex items-center justify-center rounded-md bg-gray-800 p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800" aria-controls="mobile-menu" aria-expanded="false">
              <span class="sr-only">Open main menu</span>
              //<!-- Menu open: "hidden", Menu closed: "block" -->
              <svg class="block h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
              </svg>
              //<!-- Menu open: "block", Menu closed: "hidden" -->
              <svg class="hidden h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      //<!-- Mobile menu, show/hide based on menu state. -->
      <div class="md:hidden" id="mobile-menu">
        {header_mobile()}
        <div class="border-t border-gray-700 pb-3 pt-4">
          <div class="flex items-center px-5">
            <div class="flex-shrink-0">
              <img class="h-10 w-10 rounded-full" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt=""/>
            </div>
            <div class="ml-3">
              <div class="text-base font-medium leading-none text-white">Tom Cook</div>
              <div class="text-sm font-medium leading-none text-gray-400">tom@example.com</div>
            </div>
            <button type="button" class="ml-auto flex-shrink-0 rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800">
              <span class="sr-only">View notifications</span>
              <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0" />
              </svg>
            </button>
          </div>
          <div class="mt-3 space-y-1 px-2">
            <a href="#" class="block rounded-md px-3 py-2 text-base font-medium text-gray-400 hover:bg-gray-700 hover:text-white">Your Profile</a>
            <a href="#" class="block rounded-md px-3 py-2 text-base font-medium text-gray-400 hover:bg-gray-700 hover:text-white">Settings</a>
            <a href="#" class="block rounded-md px-3 py-2 text-base font-medium text-gray-400 hover:bg-gray-700 hover:text-white">Sign out</a>
          </div>
        </div>
      </div>
    </nav>
    }
}

pub fn header_desktop() -> impl IntoView {
    let histories = format!("{}/{}", private::PRIVATE, private::HISTORIES);

    view! {
    <div class="hidden md:block">
      <div class="ml-10 flex items-baseline space-x-4">
        // <!-- Current: "bg-gray-900 text-white", Default: "text-gray-300 hover:bg-gray-700 hover:text-white" -->
        // aria-current="page"
        <a href=public::HOME      class="text-gray-300 hover:bg-gray-700 hover:text-white rounded-md px-3 py-2 text-sm font-medium">Inicio</a>
        <A href=histories         class="bg-gray-900   text-white                         rounded-md px-3 py-2 text-sm font-medium">Busqueda</A>
        <a href=public::HOME      class="text-gray-300 hover:bg-gray-700 hover:text-white rounded-md px-3 py-2 text-sm font-medium">Calendario</a>
        <a href=public::ABOUT     class="text-gray-300 hover:bg-gray-700 hover:text-white rounded-md px-3 py-2 text-sm font-medium">Acerca de nosotros</a>
      </div>
    </div>
    }
}

pub fn header_mobile() -> impl IntoView {
    let histories = format!("{}/{}", private::PRIVATE, private::HISTORIES);

    view! {
    <div class="space-y-1 px-2 pb-3 pt-2 sm:px-3">
      // <!-- Current: "bg-gray-900 text-white", Default: "text-gray-300 hover:bg-gray-700 hover:text-white" -->
      // aria-current="page"
      <a href=public::HOME      class="text-gray-300 hover:bg-gray-700 hover:text-white block rounded-md px-3 py-2 text-base font-medium" >Inicio</a>
      <a href=histories         class="bg-gray-900 text-white                           block rounded-md px-3 py-2 text-base font-medium">Busqueda</a>
      <a href=public::HOME      class="text-gray-300 hover:bg-gray-700 hover:text-white block rounded-md px-3 py-2 text-base font-medium">Calendario</a>
      <a href=public::ABOUT     class="text-gray-300 hover:bg-gray-700 hover:text-white block rounded-md px-3 py-2 text-base font-medium">Acerca de nosotros</a>
    </div>
    }
}
