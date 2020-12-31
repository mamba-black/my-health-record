import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import org.scalajs.dom

import scala.scalajs.js.{ constructorOf, | }

case class Patient(name: String)

object SearchSection {
  private val patients = Var(List[Patient]())

  def apply(): HtmlElement = {
    val searchInput = input(
      "searchInput",
      className := "form-control",
      placeholder := "Nombre del paciente",
      autoFocus(true),
    )
    val searchButton = button("buscar", className := "btn btn-outline-primary")
    val result = div(idAttr := "result")
    val eventBus = new EventBus[dom.html.Element]

    //    val obs = Observer[dom.MouseEvent | dom.KeyboardEvent](onNext = { event =>
    //      scribe.info(s"event $event")
    //      scribe.info(searchInput.ref.value)
    //    })
    val obs = Observer[dom.MouseEvent | dom.KeyboardEvent](onNext = { event =>
      val text = searchInput.ref.value
      val search = event match {
        case mouseEvent: dom.MouseEvent => scribe.debug(s"mouseEvent: $mouseEvent"); !text.trim.isEmpty
        case keyboardEvent: dom.KeyboardEvent => {
          scribe.debug(s"keyboardEvent: ${keyboardEvent.keyCode}")
          text.trim.nonEmpty
        }
        case _ => scribe.debug(s"event: $event"); false
      }
      if (search) {
        scribe.info("Buscar!")
        eventBus.writer.onNext(div(text).ref)
        patients.update(_ :+ Patient(text))
      }
    })

    searchButton.amend(onClick --> obs)
    searchInput.amend(onKeyPress.filter(e => e.keyCode == dom.ext.KeyCode.Enter) --> obs)
    //    searchInput.amend(inContext { thisNode => onInput.map(_ => thisNode.ref.value) --> obs } )
    //    searchInput.amend(inContext { thisNode => onKeyPress.map(_ => thisNode.ref.value) --> obs } )

    val a = patients.signal.map(e => e.takeRight(1).map(i => div(i.name)))
    section(
      div(
        className := "container input-group mb-3",
        searchInput,
        searchButton,
      ),
      br(),
//      child <-- eventBus.events,
      table(
        className := "container table table-hover",
        tbody(
          children <-- patients.signal.map(e => e.map(i => tr(td(i.name, onClick --> Observer[dom.MouseEvent](onNext = { event => scribe.info(s"event: ${event}") }))))),
        )
      ),
    )
  }
}


