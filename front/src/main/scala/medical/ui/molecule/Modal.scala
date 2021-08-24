package medical.ui.molecule

import com.raquo.laminar.api.L._
import medical.ui.atom.{ Button, ButtonAlt, ButtonShareStatus, CloseSvg, Two, Zero }
import org.scalajs.dom

object Modal {
  def apply(text: String, onCancel: () => Unit, onAccept: () => Unit): HtmlElement = {
    val _discard = Var[ButtonShareStatus](Zero)
    val _save = Var[ButtonShareStatus](Zero)

    div(
      cls := "flex items-center justify-center fixed left-0 bottom-0 w-full h-full bg-gray-800  bg-opacity-25",
      form(
        inContext(
          thisForm =>
            onSubmit --> Observer[dom.Event](e => {
              e.preventDefault()
              if (_discard.now() == Two) {
                thisForm.ref.parentElement.parentElement.removeChild(thisForm.ref.parentElement)
                onCancel()
              } else {
                thisForm.ref.parentElement.parentElement.removeChild(thisForm.ref.parentElement)
                onAccept()
              }
              ()
            })
        ),
        cls := "bg-white rounded-lg w-1/2",
        div(
          cls := "flex flex-col items-start p-4",
          div(
            cls := "flex items-center w-full",
            div(cls := "text-gray-900 font-medium text-lg", "My modal title", CloseSvg())
          ),
          hr(),
          div(text),
          hr(),
          div(cls := "ml-auto space-x-2", Button("Guardar", _save.writer), ButtonAlt("Descartar", _discard.writer))
        )
      )
    )
  }
}
