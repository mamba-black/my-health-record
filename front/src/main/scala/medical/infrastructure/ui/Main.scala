package medical.infrastructure.ui

import com.raquo.laminar.api.L._
import io.frontroute.LinkHandler
import org.scalajs.dom
import scribe._

object Main {

  def main(args: Array[String]): Unit = {
    debug("Test")

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
