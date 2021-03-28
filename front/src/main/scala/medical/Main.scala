package medical

import com.raquo.laminar.api.L._
import io.frontroute._
import medical.ui.component.MainUI
import org.scalajs.dom
import wvlet.log.LogSupport

object Main extends LogSupport {
  def main(args: Array[String]): Unit = {
    println("Test")

    documentEvents.onDomContentLoaded.foreach { _ =>
      LinkHandler.install()
      render(dom.document.body, MainUI())
      ()
    }(unsafeWindowOwner)
    ()
  }
}

