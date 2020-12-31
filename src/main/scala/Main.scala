import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLElement

object Main {
  def main(args: Array[String]): Unit = {
    println("Test")
    //    dom.window.addEventListener("load", init)
    documentEvents.onDomContentLoaded.foreach { _ =>
      dom.document.querySelector("html").removeChild(dom.document.querySelector("body"))
//      dom.document.querySelector("html").removeChild(dom.document.getElementById("delete"))
      render(dom.document.querySelector("html"), init())
    }(unsafeWindowOwner)
  }

  def init(): ReactiveHtmlElement[HTMLElement] = {
    body(
      header(h1("Historias Medicas")),
      br(),
      SearchSection(),
      className := "container-fluid"
    )
  }
}
