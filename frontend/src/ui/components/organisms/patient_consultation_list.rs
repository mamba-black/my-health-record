use crate::domain::appointment::State::InProgress;
use crate::domain::appointment::*;
use crate::domain::patient::Patient;
use chrono::format::Numeric::Month;
use chrono::prelude::*;
use chrono::{DateTime, Datelike, Local, Locale, TimeZone};
use chrono_tz::America::Lima;
use leptos::*;
use leptos_router::ToHref;
use ulid::Ulid;

#[component]
pub fn PatientConsultationList(patient: Patient) -> impl IntoView {
    use chrono::prelude::*;
    let date = Utc.with_ymd_and_hms(2019, 10, 28, 9, 10, 11).unwrap();
    // `2019-10-28T09:10:11Z`
    let month = Month::try_from(u8::try_from(date.month()).unwrap()).ok();

    let (aa, bb) = create_signal(vec![
        AppointmentBuilder::default()
            .patient_id(Ulid::new().to_string())
            .date(Local::now().with_timezone(&Lima))
            .state(InProgress)
            .build()
            .expect("ERROR AQUI"),
        AppointmentBuilder::default()
            .patient_id(Ulid::new().to_string())
            .date(Local::now().with_timezone(&Lima))
            .state(InProgress)
            .build()
            .expect("ERROR AQUI"),
        AppointmentBuilder::default()
            .patient_id(Ulid::new().to_string())
            .date(Local::now().with_timezone(&Lima))
            .state(InProgress)
            .build()
            .expect("ERROR AQUI"),
    ]);

    view! {
        <div class="lg:wa-7/12 lg:justify-around">
            <form>
                <ul role="list" class="divide-y divide-gray-100">
                    <For
                        each=aa
                        key=|appointment| appointment.patient_id.clone()
                        children= move |appointment: Appointment| {
                            // let patient1 = patient.clone();
                            // let full_name = patient.full_name();
                            let month = Month::try_from(u8::try_from(appointment.date.month()).unwrap()).unwrap();
                            view!{
                                <li>
                                    <a
                                        class="flex justify-between gap-x-6 py-5">
                                        // href={private::HISTORY_DETAIL.replace(":id", &patient.id)}>
                                        <div class="flex gap-x-4">

                                            <div class="flex-col justify-center items-center rounded-lg bg-white overflow-hidden shadow-md w-20">
                                              <div class="bg-blue-500 text-white py-1 px-1">
                                                // <p class="text-xs font-semibold text-white uppercase tracking-wide text-center" js-month>{month.name()}</p>
                                                <p class="text-xs font-semibold text-white uppercase tracking-wide text-center" js-month>{appointment.date.format_localized("%B", Locale::es_ES).to_string()}</p>
                                              </div>
                                              <div class="flex-col justify-center items-center">
                                                <p class="text-xs text-gray-400 text-center pt-1 px-4 leading-none" js-dayOfWeek>{appointment.date.format_localized("%A", Locale::es_ES).to_string()}</p>
                                                <p class="font-bold text-black text-center pb-1.5 px-3 leading-none" style="font-size: 18px;" js-date>{appointment.date.day()}</p>
                                              </div>
                                            </div>


                                            <div class="min-w-0 flex-auto">
                                                <p class="text-sm font-semibold leading-6 text-gray-900">
                                                    Hector Miuler Malpcia Gallegos

                                                    <span class="relative inline-block px-2 py-0 font-semibold text-green-900 leading-tight">
                                                        <span  class="absolute inset-0 bg-green-200 opacity-50 rounded-full"></span>
                                                        {
                                                            if appointment.date > Local::now() {
                                                                view!{<span class="relative">Cita programada</span>}
                                                            } else {
                                                                view!{<span></span>}
                                                            }
                                                        }
                                                    </span>
                                                </p>
                                                <p class="mt-1 truncate text-xs leading-5 text-gray-500">
                                                    miuler@gmail.com
                                                </p>
                                            </div>
                                        </div>
                                        <div class="hidden sm:flex sm:flex-col sm:items-end">
                                            // <p class="text-sm leading-6 text-gray-900">{}</p>
                                            // {
                                                // if patient.online {view!{<>
                                                //     <div class="mt-1 flex items-center gap-x-1.5">
                                                //         <div class="flex-none rounded-full bg-emerald-500/20 p-1">
                                                //             <div class="h-1.5 w-1.5 rounded-full bg-emerald-500"></div>
                                                //         </div>
                                                //         <p class="text-xs leading-5 text-gray-500">Online</p>
                                                //     </div>
                                                // </>}} else {view!{<>
                                                    <p class="mt-1 text-xs leading-5 text-gray-500">"Last seen "<time datetime="2023-01-23T13:23Z">3h ago</time></p>
                                                // </>}}
                                            // }
                                        </div>
                                    </a>
                                </li>
                            }
                        } />
                </ul>
            </form>
        </div>
    }
}
