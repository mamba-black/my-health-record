package medical

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import io.frontroute._
import medical.backend.patient.PatientReply
import medical.ui.SearchSection
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLElement
import wvlet.log.LogSupport

object Main extends LogSupport {
  def main(args: Array[String]): Unit = {
    println("Test")

    documentEvents.onDomContentLoaded.foreach { _ =>
      LinkHandler.install()
      render(dom.document.body, init())
      ()
    }(unsafeWindowOwner)
    ()
  }

  def router(patientReplyEventBus: EventBus[Option[PatientReply]]): Signal[Element] = {
    //    dom.window.addEventListener("load", init)
    val (routeResult, route) = makeRoute[Element] { render =>
      concat(
        pathEnd {
          dom.console.log("page home")
          render {
            div("HOME! 🏠")
          }
        },
        path("patient") {
          dom.console.log("page patient")
          render {
            SearchSection(patientReplyEventBus.writer)
          }
        },
        render {
          div("pageX")
        }
      )
    }
    runRoute(route, LocationProvider.browser(windowEvents.onPopState))(unsafeWindowOwner)
    BrowserNavigation.emitPopStateEvent()
    routeResult.map(_.getOrElse(div("initializing...")))
  }

  def init(): ReactiveHtmlElement[HTMLElement] = {
    info("init")

    val eventBus = new EventBus[Option[PatientReply]]
    val mainContent = div(
      className := "main",
      child <-- router(eventBus),
//      child <-- eventBus.events.observable.map(xx => PatientSection(xx.get)),
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
          li(className := "nav-item", a(href := "/", className := "nav-link active", "Home")),
          li(className := "nav-item", a(href := "/patient", className := "nav-link", "Paciente")),
//          li(className := "nav-item", a(href := "/", className := "nav-link", "Historias")),
//          li(className := "nav-item", a(href := "/", className := "nav-link", "Facturacion")),
        ),
      ),
      mainContent,
      eventBus.events --> Observer[Option[PatientReply]](onNext = { event =>
//        mainContent.ref.removeChild(searchSection.ref)
        info(s"$mainContent")
        info(s"main event: $event")
      }),
    )
  }
}
