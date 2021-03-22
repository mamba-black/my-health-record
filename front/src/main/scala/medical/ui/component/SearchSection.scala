package medical.ui.component

import com.raquo.laminar.CollectionCommand
import com.raquo.laminar.api.L._
import com.raquo.laminar.nodes.ReactiveHtmlElement
import io.grpc.stub.StreamObserver
import medical.backend.patient.{ PatientReply, PatientRequest, PatientServiceGrpcWeb }
import medical.command.{ Command, ShowPatient }
import medical.domain.Patient
import org.scalajs.dom
import org.scalajs.dom.html
import org.scalajs.dom.raw.HTMLTableCellElement
import scalapb.grpc.Channels
import scalapb.grpcweb
import wvlet.log.LogSupport

import java.util.UUID

object SearchSection extends LogSupport {

  def apply(commandWriteBus: WriteBus[Command]): HtmlElement = {
    val patientReplyEventBus = new EventBus[PatientReply]

    section(
      div(
        className := "container input-group mb-3",
//        searchButton(patientReplyEventBus, input),
      ),
      br(),
      searchTable(patientReplyEventBus, commandWriteBus),
    )
  }

  def searchInput(eventBus: EventBus[PatientReply], patientClient: PatientServiceGrpcWeb.PatientService[grpcweb.Metadata]): ReactiveHtmlElement[html.Div] = {
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

    div(
      className := "py-6 flex flex-col justify-center sm:py-12",
      div(
        className := "pt-2 relative mx-auto text-gray-600",
        input(
          className := "border-2 border-gray-300 bg-white h-10 px-5 pr-16 rounded-lg text-sm focus:outline-none",
          typ := "search",
          name := "search",
          placeholder := "Nombre del paciente",
          "searchInput",
          autoFocus(true),
          inContext(thisNode => onKeyPress.filter(e => e.keyCode == dom.ext.KeyCode.Enter) --> obsInput(thisNode)),
        ),
        button(
          typ := "submit",
          className := "absolute right-0 top-0 mt-5 mr-4",
        )
      )
    )
  }
/*
    <div class="min-h-screen bg-gray-100 py-6 flex flex-col justify-center sm:py-12">
          <div class="pt-2 relative mx-auto text-gray-600">

            <input class="border-2 border-gray-300 bg-white h-10 px-5 pr-16 rounded-lg text-sm focus:outline-none"
              type="search" name="search" placeholder="Search">
            <button type="submit" class="absolute right-0 top-0 mt-5 mr-4">
              <svg class="text-gray-600 h-4 w-4 fill-current" xmlns="http://www.w3.org/2000/svg"
                xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" id="Capa_1" x="0px" y="0px"
                viewBox="0 0 56.966 56.966" style="enable-background:new 0 0 56.966 56.966;" xml:space="preserve"
                width="512px" height="512px">
                <path
                  d="M55.146,51.887L41.588,37.786c3.486-4.144,5.396-9.358,5.396-14.786c0-12.682-10.318-23-23-23s-23,10.318-23,23  s10.318,23,23,23c4.761,0,9.298-1.436,13.177-4.162l13.661,14.208c0.571,0.593,1.339,0.92,2.162,0.92  c0.779,0,1.518-0.297,2.079-0.837C56.255,54.982,56.293,53.08,55.146,51.887z M23.984,6c9.374,0,17,7.626,17,17s-7.626,17-17,17  s-17-7.626-17-17S14.61,6,23.984,6z" />
              </svg>
            </button>
          </div>
    </div>
*/

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

  def searchTable(eventBus: EventBus[PatientReply], commandWriteBus: WriteBus[Command]): ReactiveHtmlElement[html.Div] = {
    val stub = PatientServiceGrpcWeb.stub(Channels.grpcwebChannel("https://192.168.1.3:8080"))
    val input =
      searchInput(eventBus, stub)

    div(
      className := "max-w-7xl mx-auto sm:px-6 lg:px-8", // FIXME saque el hidden
      cls.toggle("hidden") <-- eventBus.events.map(_ => {
        info("hidden:"); false
      }),
      div(
        className := "flex flex-col",
        div(
          className := "-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8",
          div(
            className := "py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8",
            input,
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
                  children.command <-- eventBus.events.map(p => CollectionCommand.Append(_td(p, commandWriteBus))),
                ),
              )
            )
          )
        ),
      )
    )
  }

  private def _td(patientReply: PatientReply, commandWriteBus: WriteBus[Command]) = {

    val obsHistory = Observer[dom.MouseEvent](onNext = { event =>
      info(s"event: ${event}")
      info(s"1: ${event.target.isInstanceOf[HTMLTableCellElement]}")
      commandWriteBus.onNext(ShowPatient(Patient(UUID.randomUUID().toString, patientReply.name)))
      ()
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
