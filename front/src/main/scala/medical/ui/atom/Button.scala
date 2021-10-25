package medical.ui.atom

import com.raquo.laminar.api.L.*
import org.scalajs.dom.MouseEvent
import scribe.*

object Button {

  def apply(text: Signal[String], callback: (ButtonShare.ButtonShareStatus, MouseEvent) => Boolean): HtmlElement = {
    ButtonShare._button(text, callback)
  }

}

object ButtonAlt {

  def apply(text: Signal[String], callback: (ButtonShare.ButtonShareStatus, MouseEvent) => Boolean): HtmlElement = {
    ButtonShare._button(text, callback, true)
  }

}

object ButtonShare {
  //def _button(text: Signal[String], toggle: Observer[ButtonShareStatus], alt: Boolean = false) = {
  private[atom] def _button(
      text: Signal[String],
      callback: (ButtonShareStatus, MouseEvent) => Boolean,
      alt: Boolean = false,
  ) = {
    debug(s"text: $text")

    val cssShare = "py-2 px-4 rounded inline-flex items-center w-36"
    val classname = if (alt) {
      s"bg-transparent hover:bg-gray-500 text-blue-700 font-semibold border border-blue-500 hover:text-white hover:border-transparent $cssShare"
    } else {
      s"bg-blue-500 hover:bg-blue-700 text-white font-bold $cssShare"
    }
    val status = Var[ButtonShareStatus](Zero)

    button(
      cls := classname,
      img(src := "/public/Education_(434).webp", cls := "w-4 mr-2"),
      child <-- text.map(span(_)),
      onClick --> Observer[MouseEvent](onNext = e => {
        val _status = status.now()
        debug(s"onClick ${_status}")
        _status match {
          case Zero =>
            callback(One, e)
            status.set(One)
            e.preventDefault()
          case One =>
            if (callback(Two, e)) status.set(Zero)
          case _ =>
            e.preventDefault()
            println("other status")
        }
        debug(s"toggleVar: onClick")
      }),
    )
  }

  sealed trait ButtonShareStatus {}
  object Zero extends ButtonShareStatus
  object One extends ButtonShareStatus
  object Two extends ButtonShareStatus
}
