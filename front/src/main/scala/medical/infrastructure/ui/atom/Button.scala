package medical.infrastructure.ui.atom

import com.raquo.laminar.api.L._
import org.scalajs.dom.MouseEvent
import scribe._

object Button {

  def apply(text: String, toggleVar: Var[Boolean], textToggle: String = null): HtmlElement = {

    input(
      `type` := "submit",
      cls := "bg-blue-400 hover:bg-blue-500 text-white font-bold py-2 px-4 rounded inline-flex items-center w-32",
      value := text,
      img(src := "/public/Education_(434).webp", cls := "w-4 mr-2"),
      onClick --> Observer[MouseEvent](onNext = _ => {
        debug("onClick")
        toggleVar.set(!toggleVar.now())
      }),
      inContext(_input => toggleVar --> Observer[Boolean](toggle => {
        if (!toggle && textToggle != null) {
          _input.ref.value = textToggle
        } else {
          _input.ref.value = text
        }
      }))
    )
  }
}
