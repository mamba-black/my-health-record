package medical.ui

import com.raquo.laminar.api.L.*
import scribe.*
import io.frontroute.LinkHandler
import medical.infrastructure.ui.MainUI
import medical.ui.component.molecule.LoginGoogleOpenIdConnect
import org.scalajs.dom

object Main {

  def main(args: Array[String]): Unit = {
    debug("Test")

    documentEvents.onDomContentLoaded.foreach { _ =>
      val session = dom.window.localStorage.getItem("session")
//      if (session != null && session.nonEmpty) {
        LinkHandler.install()
        render(dom.document.body, MainUI())
//      } else {
//        warn("No hay session, hay que loggearse")
//        render(
//          dom.document.body,
//          LoginGoogleOpenIdConnect(),
//        )
//      }
      ()
    }(unsafeWindowOwner)
    ()
  }
}
