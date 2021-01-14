package medical

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLElement

object Main {
  def main(args: Array[String]): Unit = {
    println("Test")
    //    dom.window.addEventListener("load", init)
    documentEvents.onDomContentLoaded.foreach { _ =>
//      dom.document.querySelector("html").removeChild(dom.document.querySelector("body"))
//      render(dom.document.querySelector("html"), init())
      render(dom.document.body, init())
    }(unsafeWindowOwner)
  }

  def init(): ReactiveHtmlElement[HTMLElement] = {

    val eventBus = new EventBus[Option[PatientBasic]]
    val searchSection = SearchSection(eventBus.writer)
    val mainContent = div(
      className := "main",
      searchSection,
      child <-- eventBus.events.observable.map(xx => PatientSection(xx.get)),
    )

    div(
      className := "wrapper",
      header(
        className := "d-flex flex-column flex-md-row align-items-center p-3 px-md-4 mb-3 bg-white border-bottom shadow-sm",
        h1("Historias Medicas", className := "my-0 me-md-auto fw-normal"),
        nav(
          className := "my-2 my-md-0 me-md-3",
          a("test", className := "p-2 text-dark"),
        ),
        a("Entrar", className := "btn btn-outline-primary"),
      ),
      nav(
        ul(
          className := "nav flex-column",
          li(className := "nav-item", a(className := "nav-link active", "Home")),
          li(className := "nav-item", a(className := "nav-link", "Paciente")),
          li(className := "nav-item", a(className := "nav-link", "Historias")),
          li(className := "nav-item", a(className := "nav-link", "Facturacion")),
        ),
      ),
      mainContent,
      eventBus.events --> Observer[Option[medical.PatientBasic]](onNext = { event =>
        mainContent.ref.removeChild(searchSection.ref)
        scribe.info(s"$mainContent")
        scribe.info(s"main event: $event")}
      ),
    )
  }
}
