package medical.ui

import com.raquo.laminar.api.L.*
import io.frontroute.LinkHandler
import medical.ui.molecule.LoginGoogleOpenIdConnect
import org.scalajs.dom
import scribe.*

object Main {

  def main(args: Array[String]): Unit = {
    debug("Test1")
    scribe.Logger.root
      .clearHandlers()
      .clearModifiers()
      .withHandler(minimumLevel = Some(Level.Debug))
      .replace()
    debug("Test2")

    documentEvents.onDomContentLoaded.foreach { _ =>
      val session = dom.window.localStorage.getItem("session")
      debug(s"session: $session")
      if (session.nonEmpty) {
        LinkHandler.install()
        render(dom.document.body, MainUI())
      } else {
        warn("\uD83D\uDDDE️ No hay session, hay que loggearse \uD83D\uDDDE️")
        render(dom.document.body, LoginGoogleOpenIdConnect())
      }
      ()
    }(unsafeWindowOwner)
    ()
  }
}
