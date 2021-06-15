package medical.ui

import com.raquo.laminar.api.L._
import io.frontroute._
import medical.domain.Patient
import medical.ui.atom.Loading
import medical.ui.command.CommandBus
import medical.ui.organism.{ PatientSection, SearchSection }
import org.scalajs.dom

object Router {
  def apply(commandBus: CommandBus): Signal[Element] = {

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
            SearchSection(commandBus.writer)
          }
        },
        (path("patient" / segment) & historyState) { (patientId, state) =>
          scribe.debug(s"patientId: $patientId")
          scribe.debug(s"state: $state")
          render {
            PatientSection(patientId, state.asInstanceOf[Option[Patient]])
          }
        },
        path("test") {
          render {
            div(Loading())
          }
        },
        render {
          div("pageX")
        },
      )
    }
    runRoute(route, LocationProvider.browser(windowEvents.onPopState))(unsafeWindowOwner)
    BrowserNavigation.emitPopStateEvent()
    routeResult.map(_.getOrElse(div("initializing...")))
  }
}
