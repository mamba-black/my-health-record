package medical.ui

import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import io.grpc.stub.StreamObserver
import medical.backend.patient.{ PatientClientGrpcWeb, PatientReply, PatientRequest }
import scalapb.grpc.Channels
import org.scalajs.dom
import org.scalajs.dom.html
import org.scalajs.dom.html.Table
import org.scalajs.dom.raw.HTMLTableCellElement
import scalapb.grpcweb

object SearchSection {
  private val patients = Var(List[PatientReply]())

  def apply(patientBasicWriteBus: WriteBus[Option[PatientReply]]): HtmlElement = {
    val patientNameEventBus = new EventBus[dom.html.Element]
    val stub = PatientClientGrpcWeb.stub(Channels.grpcwebChannel("http://localhost:8080"))
    val input = searchInput(patientNameEventBus, stub)

    section(
      div(
        className := "container input-group mb-3",
        input,
        searchButton(patientNameEventBus, input),
      ),
      br(),
      searchTable(patientNameEventBus, patientBasicWriteBus),
    )
  }

  def searchInput(eventBus: EventBus[dom.html.Element], patientClient: PatientClientGrpcWeb.PatientClient[grpcweb.Metadata]): ReactiveHtmlElement[html.Input] = {
    def obsInput(input: Input) = Observer[dom.KeyboardEvent](onNext = { event =>
      val text = input.ref.value
      val search = text.trim.nonEmpty
      scribe.info("Buscar!")
      if (search) {
        eventBus.writer.onNext(div(text).ref)
        patientClient.find(PatientRequest(text), new StreamObserver[PatientReply] {
          override def onNext(value: PatientReply): Unit = patients.update(_ :+ value)

          override def onError(throwable: Throwable): Unit = println("onError")

          override def onCompleted(): Unit = println("onCompleted")
        })
        patients.update(_ :+ PatientReply(text, text))
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
        patients.update(_ :+ PatientReply(text, text))
      }
    })

    button(
      "buscar",
      className := "btn btn-outline-primary",
      onClick --> obsButton,
    )
  }

  def searchTable(eventBus: EventBus[dom.html.Element], patientBasicWriteBus: WriteBus[Option[PatientReply]]): ReactiveHtmlElement[html.Div] = {
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

    div(
      className := "max-w-7xl mx-auto sm:px-6 lg:px-8 hidden",
      cls.toggle("hidden") <-- eventBus.events.map(_ => false),
      div(
        className := "flex flex-col",
        div(
          className := "-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8",
          div(
            className := "py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8",
            div(
              className := "shadow overflow-hidden border-b border-gray-200 sm:rounded-lg",
              table(
                className := "min-w-full divide-y divide-gray-200",
                thead(
                  className := "bg-gray-50",
                  tr(
                    th(
                      className := "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                      "Nombre",
                    ),
                  )
                ),
                tbody(
                  className := "bg-white divide-y divide-gray-200",
                  children <-- patients.signal.map(e => e.zipWithIndex.map(i => {
//                    tr(td(idAttr := s"${i._2}", i._1.name, onClick --> obsHistory))
                    tr(
                      td(
                        className := "px-6 py-4 whitespace-nowrap",
                        div(
                          className := "flex items-center",
                          div(
                            className := "flex-shrink-0 h-10 w-10",
                            img(cls := "h-10 w-10 rounded-full", src := "https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&amp;ixid=eyJhcHBfaWQiOjEyMDd9&amp;auto=format&amp;fit=facearea&amp;facepad=4&amp;w=256&amp;h=256&amp;q=60")
                          ),
                          div(
                            cls := "ml-4",
                            div(
                              cls := "text-2xl font-medium font-semibold text-gray-900",
                              i._1.name,
                            ),
                            div(
                              cls := "text-sm text-gray-500",
                              "test",
                            ),
                          )
                        ),
                        onClick --> obsHistory,
                      )
                    )
                  })),
                ),
              )
            )
          )
        ),
      )
    )
  }
}


