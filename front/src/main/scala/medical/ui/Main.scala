package medical.ui

import com.raquo.laminar.api.L.*
import io.frontroute.LinkHandler
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
