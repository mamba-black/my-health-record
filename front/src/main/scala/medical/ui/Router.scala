package medical.ui

import com.raquo.laminar.api.L._
import io.frontroute.{ BrowserNavigation, LocationProvider, concat, makeRoute, path, pathEnd, runRoute }
import medical.domain.Patient
import org.scalajs.dom

object Router {
  def apply(patientEventBus: EventBus[Option[Patient]]): Signal[Element] = {

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
            SearchSection(patientEventBus.writer)
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
