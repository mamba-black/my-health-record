package medical.ui

import com.raquo.laminar.api.L._
import io.frontroute.LinkHandler
import medical.ui.component.MainUI
import medical.ui.component.molecule.LoginGoogleOpenIdConnect
import org.scalajs.dom
import wvlet.log.{ LogLevel, LogSupport }

object Main extends LogSupport {
  wvlet.log.Logger.setDefaultLogLevel(LogLevel.DEBUG)

  def main(args: Array[String]): Unit = {
    println("Test")

    documentEvents.onDomContentLoaded.foreach { _ =>
      val session = dom.window.localStorage.getItem("session")
      if (session != null && session.nonEmpty) {
        LinkHandler.install()
        render(dom.document.body, MainUI())
      } else {
        warn("No hay session, hay que loggearse")
        render(
          dom.document.body,
          LoginGoogleOpenIdConnect(),
        )
      }
      ()
    }(unsafeWindowOwner)
    ()
  }
}
