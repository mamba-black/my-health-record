package medical.infrastructure.ui.molecule

import com.raquo.laminar.api.L._
import medical.infrastructure.ui.atom.{ Button, ButtonAlt, CloseSvg }
import scribe.info

object Modal {
  def apply(text: String, toggleVar: Var[Boolean], showModal: Var[Boolean]): HtmlElement = {
    val discard = Var[Boolean](false)
    val save = Var[Boolean](false)

    div(
      cls := "flex items-center justify-center fixed left-0 bottom-0 w-full h-full bg-gray-800  bg-opacity-25",
      inContext(modal => discard --> Observer[Boolean](_discard => {
        toggleVar.set(!toggleVar.now())
//        if(_discard) modal.ref.parentElement.removeChild(modal.ref)
        info(s"$modal")
        if(_discard) showModal.set(!showModal.now())
        ()
      })),
      div(
        cls := "bg-white rounded-lg w-1/2",
        div(
          cls := "flex flex-col items-start p-4",
          div(
            cls := "flex items-center w-full",
            div(
              cls := "text-gray-900 font-medium text-lg",
              "My modal title",
              CloseSvg(),
            ),
          ),
          hr(),
          div(text),
          hr(),
          div(
            cls := "ml-auto space-x-2",
            Button("Guardar", save),
            ButtonAlt("Descartar", discard),
          ),
        ),
      ),
    )
  }
}

