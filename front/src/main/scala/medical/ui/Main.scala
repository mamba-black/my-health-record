package medical.ui

import com.raquo.laminar.api.L.*
import io.frontroute.LinkHandler
import medical.ui.component.MainUI
import medical.ui.component.molecule.LoginGoogleOpenIdConnect
import org.scalajs.dom
import scribe.*

object Main {
  val t: Test = new TestImpl();
  t.uno()

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


trait Test {
  def uno(): String
}

class TestImpl extends Test {
  def uno(): String = {
    "uno"
  }
}
