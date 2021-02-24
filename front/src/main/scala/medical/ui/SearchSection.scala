package medical.ui

import com.raquo.laminar.CollectionCommand
import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import io.grpc.stub.StreamObserver
import medical.backend.patient.{ PatientReply, PatientRequest, PatientServiceGrpcWeb }
import org.scalajs.dom
import org.scalajs.dom.html
import org.scalajs.dom.raw.HTMLTableCellElement
import scalapb.grpc.Channels
import scalapb.grpcweb
import wvlet.log.LogSupport

object SearchSection extends LogSupport {

  def apply(patientBasicWriteBus: WriteBus[Option[PatientReply]]): HtmlElement = {
    val patientReplyEventBus = new EventBus[PatientReply]
    val stub = PatientServiceGrpcWeb.stub(Channels.grpcwebChannel("https://192.168.1.3:8080"))
    val input = searchInput(patientReplyEventBus, stub)

    section(
      div(
        className := "container input-group mb-3",
        input,
        searchButton(patientReplyEventBus, input),
      ),
      br(),
      searchTable(patientReplyEventBus, patientBasicWriteBus),
    )
  }

  def searchInput(eventBus: EventBus[PatientReply], patientClient: PatientServiceGrpcWeb.PatientService[grpcweb.Metadata]): ReactiveHtmlElement[html.Input] = {
    def obsInput(input: Input) = Observer[dom.KeyboardEvent](onNext = { _ =>
      val text = input.ref.value
      val search = text.trim.nonEmpty
      if (search) {
        info("Buscar!")
        eventBus.writer.onNext(PatientReply("-2", text))
        patientClient.find(PatientRequest(text), new StreamObserver[PatientReply] with LogSupport {
          override def onNext(value: PatientReply): Unit = info(s"patient: $value") // patients.update(_ :+ value)

          override def onError(throwable: Throwable): Unit = warn("onError")

          override def onCompleted(): Unit = info("onCompleted")
        })
      }
      ()
    })

    input(
      "searchInput",
      className := "form-control",
      placeholder := "Nombre del paciente",
      autoFocus(true),
      inContext(thisNode => onKeyPress.filter(e => e.keyCode == dom.ext.KeyCode.Enter) --> obsInput(thisNode))
    )
  }

  def searchButton(eventBus: EventBus[PatientReply], input: ReactiveHtmlElement[html.Input]): ReactiveHtmlElement[html.Button] = {

    val obsButton = Observer[dom.MouseEvent](onNext = { event =>
      info(s"mouseEvent: $event")
      val text = input.ref.value
      if (text.trim.nonEmpty) {
        info(s"Buscar! $text")
        eventBus.writer.onNext(PatientReply("-3", text))
      }
    })

    button(
      "buscar",
      className := "btn btn-outline-primary",
      onClick --> obsButton,
    )
  }

  def searchTable(eventBus: EventBus[PatientReply], patientBasicWriteBus: WriteBus[Option[PatientReply]]): ReactiveHtmlElement[html.Div] = {

    div(
      className := "max-w-7xl mx-auto sm:px-6 lg:px-8", // FIXME saque el hidden
      cls.toggle("hidden") <-- eventBus.events.map(_ => { info("hidden:") ; false }),
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
                  //child <-- eventBus.events.map(e => { _td(e, eventBus, patientBasicWriteBus) })
                  children.command <-- eventBus.events.map(p => CollectionCommand.Append(_td(p, eventBus, patientBasicWriteBus)) ),
                ),
              )
            )
          )
        ),
      )
    )
  }

  private def _td(patientReply: PatientReply, eventBus: EventBus[PatientReply], patientBasicWriteBus: WriteBus[Option[PatientReply]]) = {

    val obsHistory = Observer[dom.MouseEvent](onNext = { event =>
      info(s"event: ${event}")
      info(s"1: ${event.target.isInstanceOf[HTMLTableCellElement]}")
      val _ = eventBus.events --> (e => patientBasicWriteBus.onNext(Some(e)))
    })

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
              patientReply.name,
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
  }
}


