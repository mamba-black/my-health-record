package medical.ui.atom

import com.raquo.laminar.api.L._
import org.scalajs.dom.MouseEvent
import scribe._

object Button {

  def apply(text: String, toggle: Observer[ButtonShareStatus], textToggle: String = null): HtmlElement = {
    ButtonShare.button(text, toggle, textToggle)
  }

}

object ButtonAlt {

  def apply(text: String, toggle: Observer[ButtonShareStatus], textToggle: String = null): HtmlElement = {
    ButtonShare.button(text, toggle, textToggle, true)
  }

}

private[atom] object ButtonShare {
  def button(text: String, toggle: Observer[ButtonShareStatus], textToggle: String, alt: Boolean = false) = {

    val cssShare = "py-2 px-4 rounded inline-flex items-center w-28"
    val classname = if (alt) {
      s"bg-transparent hover:bg-gray-500 text-blue-700 font-semibold border border-blue-500 hover:text-white hover:border-transparent $cssShare"
    } else {
      s"bg-blue-500 hover:bg-blue-700 text-white font-bold $cssShare"
    }
    val status = Var[ButtonShareStatus](Zero)

    input(
      `type` := "button",
      cls := classname,
      value := text,
      img(src := "/public/Education_(434).webp", cls := "w-4 mr-2"),
      onClick --> Observer[MouseEvent](onNext = _ => {
        val _status = status.now()
        debug(s"onClick ${_status}")
        _status match {
          case Zero =>
            toggle.onNext(One)
            if (textToggle != null) status.set(One)
          case One =>
            toggle.onNext(Two)
            status.set(Zero)
          case _ =>
            println("other status")
        }
        debug(s"toggleVar: onClick")
      }),
      inContext(_input => status.signal --> Observer[ButtonShareStatus](toggle => {
        if (textToggle != null) {
          toggle match {
            case Zero =>
              _input.ref.value = text
            case One =>
              _input.ref.value = textToggle
            case _ =>
              println("other status")
          }
        }
      }))
    )
  }
}

sealed trait ButtonShareStatus {}
object Zero extends ButtonShareStatus
object One extends ButtonShareStatus
object Two extends ButtonShareStatus
