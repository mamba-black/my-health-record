package medical.ui.molecule

import com.raquo.laminar.api.L.*
import medical.ui.atom.{ Button, ButtonAlt, CloseSvg }
import org.scalajs.dom.MouseEvent
import scribe.debug

object Modal {
  def apply(text: String, onCancel: () => Unit, onAccept: () => Unit, onClose: () => Unit): HtmlElement = {

    div(
      cls := "fixed inset-0 z-10 overflow-y-auto",
      div(
        cls := "fixed inset-0 w-full h-full bg-black opacity-75",
        inContext(dialog => {
          onClick --> Observer[MouseEvent](onNext = e => {
            dialog.ref.parentElement.parentElement.removeChild(dialog.ref.parentElement)
            onClose()
          })
        }),
      ),
      inContext(dialog => {
        div(
          cls := "flex items-center min-h-screen px-4 py-8",
          div(
            cls := "relative w-full max-w-lg p-4 mx-auto bg-white rounded-md shadow-lg",
            div(
              cls := "mt-3",
              div(),
              div(
                cls := "mt-2 text-center",
                h4(cls := "text-lg font-medium text-gray-800", text),
                p(cls := "mt-2 text-[15px] leading-relaxed text-gray-500", "..."),
              ),
            ),
            div(
              cls := "items-center gap-2 mt-3 sm:flex",
              Button(
                Signal.fromValue("Guardar"),
                _ => {
                  debug("Guardar")
                  onAccept()
                  dialog.ref.parentElement.parentElement.removeChild(dialog.ref.parentElement)
                  true
                },
              ),
              ButtonAlt(
                Signal.fromValue("Descartar"),
                _ => {
                  debug("Descartar")
                  onCancel()
                  dialog.ref.parentElement.parentElement.removeChild(dialog.ref.parentElement)
                  true
                },
              ),
            ),
          ),
        )
      }),
    )
  }

}
