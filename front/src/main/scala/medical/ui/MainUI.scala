package medical.ui

import com.raquo.laminar.api.L._
import medical.domain.Patient
import wvlet.log.LogSupport

object MainUI extends LogSupport {
  def apply(eventBus: EventBus[Option[Patient]], signalElements: Signal[Element]): HtmlElement = {

    val mainContent = div(
      className := "main",
      child <-- signalElements,
      //      child <-- eventBus.events.observable.map(xx => PatientSection(xx.get)),
    )

    div(
      className := "wrapper",
      header(
        className := "d-flex flex-column flex-md-row align-items-center p-3 px-md-4 mb-3 bg-white border-bottom shadow-sm",
        h1("Historias Medicas", className := "my-0 me-md-auto fw-normal"),
        nav(
          className := "my-2 my-md-0 me-md-3",
          a(href := "/", "test", className := "p-2 text-dark"),
        ),
        a(href := "/patient", "Entrar", className := "btn btn-outline-primary"),
      ),
      nav(
        ul(
          className := "nav flex-column",
          li(className := "nav-item", a(href := "/", className := "nav-link active", "Home")),
          li(className := "nav-item", a(href := "/patient", className := "nav-link", "Paciente")),
          //          li(className := "nav-item", a(href := "/", className := "nav-link", "Historias")),
          //          li(className := "nav-item", a(href := "/", className := "nav-link", "Facturacion")),
        ),
      ),
      mainContent,
      eventBus.events --> Observer[Option[Patient]](onNext = { event =>
        //        mainContent.ref.removeChild(searchSection.ref)
        info(s"$mainContent")
        info(s"main event: $event")
      }),
    )
  }
}
