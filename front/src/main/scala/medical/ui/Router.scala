package medical.ui

import com.raquo.laminar.api.L.*
import io.frontroute.*
import medical.command.CommandBus
import medical.domain.Patient
import medical.ui.component.organism.{ PatientSection, SearchSection }
import org.scalajs.dom
import scribe.*

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
          info(s"patientId: $patientId")
          info(s"state: $state")
          render {
            PatientSection(patientId, state.asInstanceOf[Option[Patient]])
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
}
