package medical.ui.component

import com.raquo.laminar.api.L._
import medical.command.CommandBus
import medical.ui.CommandHandler
import wvlet.log.LogSupport

object MainUI extends LogSupport {
  def apply(commandBus: CommandBus, signalElements: Signal[Element]): HtmlElement = {

    div(
      className := "wrapper",
      _header(),
      _nav(),
      _main(signalElements),
      CommandHandler(commandBus),
    )
  }

  private def _main(signalElements: Signal[Element]): HtmlElement = {
    val mainContent = div(
      className := "main",
      child <-- signalElements,
    )
    mainContent
  }

  private def _header(): HtmlElement = {
    header(
      className := "d-flex flex-column flex-md-row align-items-center p-3 px-md-4 mb-3 bg-white border-bottom shadow-sm",
      h1("Historias Medicas", className := "my-0 me-md-auto fw-normal"),
      nav(
        className := "my-2 my-md-0 me-md-3",
        a(href := "/", "test", className := "p-2 text-dark"),
      ),
      a(href := "/patient", "Entrar", className := "btn btn-outline-primary"),
    )
  }

  private def _nav(): HtmlElement = {
    nav(
      ul(
        className := "nav flex-column",
        li(className := "nav-item", a(href := "/", className := "nav-link active", "Home")),
        li(className := "nav-item", a(href := "/patient", className := "nav-link", "Paciente")),
      ),
    )
  }
}
