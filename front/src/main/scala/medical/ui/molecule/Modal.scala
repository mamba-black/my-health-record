package medical.ui.molecule

import com.raquo.laminar.api.L.*
import medical.ui.atom.{ Button, ButtonAlt, CloseSvg }
import org.scalajs.dom.MouseEvent
import scribe.debug

object Modal {
  def apply(text: String, onCancel: () => Unit, onAccept: () => Unit): HtmlElement = {

    div(
      cls := "flex items-center justify-center fixed left-0 bottom-0 w-full h-full bg-gray-800  bg-opacity-25",
      form(
        cls := "bg-white rounded-lg w-1/2",
        inContext(thisForm => {
          div(
            cls := "flex flex-col items-start p-4",
            div(
              cls := "flex items-center w-full",
              div(cls := "text-gray-900 font-medium text-lg", "My modal title", CloseSvg()),
            ),
            hr(),
            div(text),
            hr(),
            div(
              cls := "ml-auto space-x-2",
              Button(
                Signal.fromValue("Guardar"),
                (_, _) => {
                  debug("Guardar")
                  onAccept()
                  thisForm.ref.parentElement.parentElement.removeChild(thisForm.ref.parentElement)
                  true
                },
              ),
              ButtonAlt(
                Signal.fromValue("Descartar"),
                (_, _) => {
                  debug("Descartar")
                  onCancel()
                  thisForm.ref.parentElement.parentElement.removeChild(thisForm.ref.parentElement)
                  true
                },
              ),
            ),
          )
        }),
      ),
    )

    div(
      cls := "fixed inset-0 z-10 overflow-y-auto",
      div(
        cls := "fixed inset-0 w-full h-full bg-black opacity-75",
        inContext(dialog => {
          onClick --> Observer[MouseEvent](onNext = e => {
            dialog.ref.parentElement.parentElement.removeChild(dialog.ref.parentElement)
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
                (_, _) => {
                  debug("Guardar")
                  onAccept()
                  dialog.ref.parentElement.parentElement.removeChild(dialog.ref.parentElement)
                  true
                },
              ),
              ButtonAlt(
                Signal.fromValue("Descartar"),
                (_, _) => {
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
