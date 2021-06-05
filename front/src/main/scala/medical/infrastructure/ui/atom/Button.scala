package medical.infrastructure.ui.atom

import com.raquo.laminar.api.L._
import org.scalajs.dom.MouseEvent
import scribe._

object Button {

  def apply(readOnly: Var[Boolean]): HtmlElement = {
    input(
      `type` := "submit",
      cls := "bg-blue-400 hover:bg-blue-500 text-white font-bold py-2 px-4 rounded inline-flex items-center w-32",
      img(src := "/public/Education_(434).webp", cls := "w-4 mr-2"),
      span(child.text <-- readOnly.signal.map(ro => if (ro) "Editar" else "Grabar")),
      onClick --> Observer[MouseEvent](onNext = _ => {
        debug("onClick")
        readOnly.set(!readOnly.now())
      })
    )
  }
}
