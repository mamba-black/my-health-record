package medical.ui

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import medical.dtos.PatientBasic
import org.scalajs.dom
import org.scalajs.dom.html
import org.scalajs.dom.html.Table
import org.scalajs.dom.raw.HTMLTableCellElement

object SearchSection {
  private val patients = Var(List[PatientBasic]())

  def apply(patientBasicWriteBus: WriteBus[Option[PatientBasic]]): HtmlElement = {
    val eventBus = new EventBus[dom.html.Element]
    val input = searchInput(eventBus)

    section(
      div(
        className := "container input-group mb-3",
        input,
        searchButton(eventBus, input),
      ),
      br(),
      searchTable(patientBasicWriteBus),
    )
  }

  def searchInput(eventBus: EventBus[dom.html.Element]): ReactiveHtmlElement[html.Input] = {
    def obsInput(input: Input) = Observer[dom.KeyboardEvent](onNext = { event =>
      val text = input.ref.value
      val search = text.trim.nonEmpty
      scribe.info("Buscar!")
      if (search) {
        eventBus.writer.onNext(div(text).ref)
        patients.update(_ :+ PatientBasic(text, text))
      }
    })

    input(
      "searchInput",
      className := "form-control",
      placeholder := "Nombre del paciente",
      autoFocus(true),
      inContext(thisNode => onKeyPress.filter(e => e.keyCode == dom.ext.KeyCode.Enter) --> obsInput(thisNode))
    )
  }

  def searchButton(eventBus: EventBus[dom.html.Element], input: ReactiveHtmlElement[html.Input]): ReactiveHtmlElement[html.Button] = {

    val obsButton = Observer[dom.MouseEvent](onNext = { event =>
      val text = input.ref.value
      scribe.info(s"mouseEvent: $event")
      val search = text.trim.isEmpty
      scribe.info(s"mouseEvent: $event")
      if (!search) {
        scribe.info("Buscar!")
        eventBus.writer.onNext(div(text).ref)
        patients.update(_ :+ PatientBasic(text, text))
      }
    })

    button(
      "buscar",
      className := "btn btn-outline-primary",
      onClick --> obsButton,
    )
  }

  def searchTable(patientBasicWriteBus: WriteBus[Option[PatientBasic]]): ReactiveHtmlElement[Table] = {
    val obsHistory = Observer[dom.MouseEvent](onNext = { event =>
      scribe.info(s"event: ${event}")
      scribe.info(s"1: ${event.target.isInstanceOf[HTMLTableCellElement]}")
      val target = event.target.asInstanceOf[HTMLTableCellElement]
      scribe.info(s"event.id: ${target.id}")
      val i = target.id.toInt
      val patient = patients.now()(i)
      scribe.info(s"patient.name: ${patient.name}, patient.id: ${patient.id}")
      patientBasicWriteBus.onNext(Some(patient))
    })

    table(
      className := "container table table-hover",
      tbody(
        children <-- patients.signal.map(e => e.zipWithIndex.map(i => tr(td(idAttr := s"${i._2}", i._1.name, onClick --> obsHistory)))),
      )
    )
  }
}
