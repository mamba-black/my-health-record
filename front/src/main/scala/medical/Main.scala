package medical

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import io.frontroute._
import medical.command.{ Command, CommandBus }
import medical.ui.{ Router }
import medical.ui.component.{ MainUI }
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLElement
import wvlet.log.LogSupport

object Main extends LogSupport {
  def main(args: Array[String]): Unit = {
    println("Test")

    documentEvents.onDomContentLoaded.foreach { _ =>
      LinkHandler.install()
      render(dom.document.body, init())
      ()
    }(unsafeWindowOwner)
    ()
  }

  def init(): ReactiveHtmlElement[HTMLElement] = {
    info("init")
    val commandBus: CommandBus = new EventBus[Command]
    MainUI(commandBus, Router(commandBus))
  }
}
