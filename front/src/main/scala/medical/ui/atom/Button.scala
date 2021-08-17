package medical.ui.atom

import com.raquo.laminar.api.L._
import org.scalajs.dom.MouseEvent
import scribe._

object Button {

  def apply(text: String, toggleVar: Var[Boolean], textToggle: String = null): HtmlElement = {
    ButtonShare.button(text, toggleVar, textToggle)
  }

}

object ButtonAlt {

  def apply(text: String, toggleVar: Var[Boolean], textToggle: String = null): HtmlElement = {
    ButtonShare.button(text, toggleVar, textToggle, true)
  }

}

private[atom] object ButtonShare {
  def button(text: String, toggleVar: Var[Boolean], textToggle: String, alt: Boolean = false) = {
    val cssShare = "py-2 px-4 rounded inline-flex items-center w-28"
    val classname = if (alt) {
      s"bg-transparent hover:bg-gray-500 text-blue-700 font-semibold border border-blue-500 hover:text-white hover:border-transparent $cssShare"
    } else {
      s"bg-blue-500 hover:bg-blue-700 text-white font-bold $cssShare"
    }

    input(
      `type` := "submit",
      cls := classname,
      value := text,
      img(src := "/public/Education_(434).webp", cls := "w-4 mr-2"),
      onClick --> Observer[MouseEvent](onNext = _ => {
        debug("onClick")
        toggleVar.set(!toggleVar.now())
        debug(s"toggleVar: ${toggleVar.now()}")
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
