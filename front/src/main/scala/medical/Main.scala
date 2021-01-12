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
      dom.document.querySelector("html").removeChild(dom.document.querySelector("body"))
      //      dom.document.querySelector("html").removeChild(dom.document.getElementById("delete"))
      render(dom.document.querySelector("html"), init())
    }(unsafeWindowOwner)
  }

  def init(): ReactiveHtmlElement[HTMLElement] = {

    val eventBus = new EventBus[Option[PatientBasic]]
    val searchSection = SearchSection(eventBus.writer)
    val mainContent = div(
      searchSection,
      child <-- eventBus.events.observable.map(xx => div(xx.get.name)),
    )

    body(
      header(
        className := "d-flex flex-column flex-md-row align-items-center p-3 px-md-4 mb-3 bg-white border-bottom shadow-sm",
        h1("Historias Medicas", className := "my-0 me-md-auto fw-normal"),
        nav(
          className := "my-2 my-md-0 me-md-3",
          a("test", className := "p-2 text-dark"),
        ),
        a("Entrar", className := "btn btn-outline-primary"),
      ),
      br(),
      mainContent,
      eventBus.events --> Observer[Option[medical.PatientBasic]](onNext = { event =>
        mainContent.ref.removeChild(searchSection.ref)
        scribe.info(s"$mainContent")
        scribe.info(s"main event: $event")}
      ),
    )
  }
}
