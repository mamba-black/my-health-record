package medical.infrastructure.ui

import com.raquo.laminar.api.L._
import io.frontroute._
import medical.infrastructure.ui.command.CommandBus
import medical.domain.Patient
import medical.infrastructure.ui.organism.{ PatientSection, SearchSection }
import org.scalajs.dom
import scribe._

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
