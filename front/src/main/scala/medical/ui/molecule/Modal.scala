package medical.ui.molecule

import com.raquo.laminar.api.L.*
import medical.ui.atom.{Button, ButtonAlt, CloseSvg}
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
  }

}
