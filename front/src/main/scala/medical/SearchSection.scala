package medical

import com.raquo.laminar.api.L._
import org.scalajs.dom
import org.scalajs.dom.raw.HTMLTableCellElement

import scala.scalajs.js.|

object SearchSection {
  private val patients = Var(List[PatientBasic]())

  def apply(patientBasicWriteBus: WriteBus[Option[PatientBasic]]): HtmlElement = {
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
        patients.update(_ :+ PatientBasic(text, text))
      }
    })
    val onHistory = Observer[dom.MouseEvent](onNext = {event =>
      scribe.info(s"event: ${event}")
      scribe.info(s"1: ${event.target.isInstanceOf[HTMLTableCellElement]}")
      scribe.info(s"2: ${event.target.asInstanceOf[HTMLTableCellElement]}")
      val target = event.target.asInstanceOf[HTMLTableCellElement]
      scribe.info(s"event.id: ${target.id}")
      val i = target.id.toInt
      val patient = patients.now()(i)
      scribe.info(s"patient.name: ${patient.name}, patient.id: ${patient.id}")
      patientBasicWriteBus.onNext(Some(patient))
    })

    searchButton.amend(onClick --> obs)
    searchInput.amend(onKeyPress.filter(e => e.keyCode == dom.ext.KeyCode.Enter) --> obs)
    //    searchInput.amend(inContext { thisNode => onInput.map(_ => thisNode.ref.value) --> obs } )
    //    searchInput.amend(inContext { thisNode => onKeyPress.map(_ => thisNode.ref.value) --> obs } )

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
          children <-- patients.signal.map(e => e.zipWithIndex.map(i => tr(td(idAttr := s"${i._2}", i._1.name, onClick --> onHistory)))),
        )
      ),
    )
  }
}


