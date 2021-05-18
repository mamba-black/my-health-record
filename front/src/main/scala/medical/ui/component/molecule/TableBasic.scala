package medical.ui.component.molecule

import com.raquo.laminar.api.L.*
import com.raquo.laminar.nodes.ReactiveElement.Base
import scribe.*

object TableBasic {

  def apply(tds: Option[Inserter[Base]]): HtmlElement = {

    val _tds = tds.getOrElse(children.command <-- EventStream.empty)

    div(
      cls := "max-w-7xl mx-auto sm:px-6 lg:px-8", // FIXME saque el hidden
//      cls.toggle("hidden") <-- eventBus.events.map(_ => {
//        info("hidden:")
//        false
//      }),
      div(
        cls := "flex flex-col",
        div(
          cls := "-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8",
          div(
            cls := "py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8",
            div(
              cls := "shadow overflow-hidden border-b border-gray-200 sm:rounded-lg",
              table(
                cls := "min-w-full divide-y divide-gray-200",
                thead(
                  cls := "bg-gray-50",
                  tr(
                    th(
                      cls := "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                      "Nombre",
                    ),
                  )
                ),
                tbody(
                  cls := "bg-white divide-y divide-gray-200",
                  //child <-- eventBus.events.map(e => { _td(e, eventBus, patientBasicWriteBus) })
                  _tds,
                ),
              )
            )
          )
        ),
      )
    )
  }

}
