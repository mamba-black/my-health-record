package medical.ui.atom

import com.raquo.laminar.api.L.*
import org.scalajs.dom
import org.scalajs.dom.{ CustomEventInit, MouseEvent }
import scribe.*

object Button {

  def apply(text: Signal[String], callback: MouseEvent => Boolean): HtmlElement = {
    ButtonShare(text, callback)
  }

}

object ButtonAlt {

  def apply(text: Signal[String], callback: MouseEvent => Boolean): HtmlElement = {
    ButtonShare(text, callback, true)
  }

}

object ButtonShare {
  // def _button(text: Signal[String], toggle: Observer[ButtonShareStatus], alt: Boolean = false) = {
  private[atom] def apply(text: Signal[String], callback: MouseEvent => Boolean, alt: Boolean = false) = {
    debug(s"text: $text")

    val cssShare = "py-2 px-4 rounded inline-flex items-center w-36"
    val classname = if (alt) {
      s"bg-transparent hover:bg-gray-500 text-blue-700 font-semibold border border-blue-500 hover:text-white hover:border-transparent $cssShare"
      s"w-full mt-2 p-2.5 flex-1 text-gray-800 rounded-md outline-none border ring-offset-2 ring-blue-600 focus:ring-2"
    } else {
      s"bg-blue-500 hover:bg-blue-700 text-white font-bold $cssShare"
      s"w-full mt-2 p-2.5 flex-1 text-white bg-blue-500 hover:bg-blue-700 rounded-md outline-none ring-offset-2 ring-blue-600 focus:ring-2"
    }

    button(
      cls := classname,
      // img(src := "/public/Education_(434).webp", cls := "w-4 mr-2"),
      child <-- text.map(span(_)),
      onClick --> Observer[MouseEvent](onNext = e => {
        e.preventDefault()
        debug(s"toggleVar: onClick")
        callback(e)
      }),
    )
  }

  sealed trait ButtonShareStatus {}
  object Zero extends ButtonShareStatus
  object One extends ButtonShareStatus
  object Two extends ButtonShareStatus
}
