package medical.infrastructure.ui

import com.raquo.laminar.api.L._
import medical.infrastructure.ui.command._

object MainUI {
  def apply(): HtmlElement = {
    val commandBus: CommandBus = new EventBus[Command]
    val routeSignalElement = Router(commandBus)
    val commandHandler = CommandHandler(commandBus)

    div(
      idAttr := "test001",
      mainHeader(),
      mainContent(routeSignalElement),
      commandHandler,
    )
  }

  private def mainContent(signalElements: Signal[Element]): HtmlElement = {
    val mainContent = div(
      className := "main",
      child <-- signalElements,
    )
    mainContent
  }

  private def mainHeader(): HtmlElement = {
    nav(
      cls := "w-full z-30 bg-white shadow-lg border-b border-gray-300 ",
      div(
        cls := "w-full flex items-center justify-between mt-0 px-6 py-0",
        div(
          cls := "md:flex md:items-center md:w-auto w-full order-3 md:order-1",
          nav(
            div(
              cls := "inline-flex",
              div(
                cls := "mx-auto h-10  pt-4 w-auto pr-10",
                svg.svg(
                  svg.className := "w-8 -8 text-orange", svg.viewBox := "0 0 54 54",
                  svg.path(svg.d := "M13.5 22.1c1.8-7.2 6.3-10.8 13.5-10.8 10.8 0 12.15 8.1 17.55 9.45 3.6.9 6.75-.45 9.45-4.05-1.8 7.2-6.3 10.8-13.5 10.8-10.8 0-12.15-8.1-17.55-9.45-3.6-.9-6.75.45-9.45 4.05zM0 38.3c1.8-7.2 6.3-10.8 13.5-10.8 10.8 0 12.15 8.1 17.55 9.45 3.6.9 6.75-.45 9.45-4.05-1.8 7.2-6.3 10.8-13.5 10.8-10.8 0-12.15-8.1-17.55-9.45-3.6-.9-6.75.45-9.45 4.05z"),
                ),
              ),
            ),
            div(
              cls := "inline-flex",
              ul(
                cls := "md:flex items-center justify-center text-base text-gray-400  md:pt-0",
                li(a(
                  cls := "inline-block no-underline hover:text-black font-medium text-md text-gray-800 py-2 px-4 lg:-ml-2",
                  href := "/",
                  "Home",
                )),
                li(a(
                  cls := "inline-block no-underline hover:text-gray-700 font-medium text-md py-2 px-4 lg:-ml-2",
                  href := "/patient",
                  "Pacientes",
                )),
                li(a(
                  cls := "inline-block no-underline hover:text-gray-700 font-medium text-md py-2 px-4 lg:-ml-2",
                  href := "/configuration",
                  "Configuración",
                )),
                li(a(
                  cls := "inline-block no-underline hover:text-gray-700 font-medium text-md py-2 px-4 lg:-ml-2",
                  href := "/about",
                  "About",
                )),
              ),
            ),
          )
        ),
      ),
    )
  }
}
