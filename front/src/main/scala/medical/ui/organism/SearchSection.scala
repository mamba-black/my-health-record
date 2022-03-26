package medical.ui.organism

import com.raquo.laminar.CollectionCommand
import com.raquo.laminar.api.L.*
import com.raquo.laminar.nodes.ReactiveHtmlElement
import medical.api.patientapi.{ PatientApiGrpcWeb, PatientReply }
import medical.domain.{ ContactPoint, HumanName, Patient, SystemContactPoint }
import medical.infrastructure.patientRepository
import medical.ui.command.{ Command, ShowPatient }
import medical.ui.molecule.TableBasic
import org.scalajs.dom
import org.scalajs.dom.html
import org.scalajs.dom.raw.HTMLTableCellElement
import scalapb.grpc.Channels
import scribe.*

import java.time.LocalDate

object SearchSection {

  def apply(commandWriteBus: WriteBus[Command]): HtmlElement = {
    debug("Begin")
    val patientReplyEventBus = new EventBus[PatientReply]

    val grpcUrl = "https://0.0.0.0:9443" // "http://0.0.0.0:9090"
    val patientApi = PatientApiGrpcWeb.stub(Channels.grpcwebChannel(grpcUrl))

    section(searchInput(patientReplyEventBus), searchTable(patientReplyEventBus, commandWriteBus))
  }

  private def searchInput(eventBus: EventBus[PatientReply]): ReactiveHtmlElement[html.Div] = {
    def searchName(input: Input): Unit = {
      val text = input.ref.value
      val search = text.trim.asInstanceOf[String].nonEmpty
      if (search) {
        debug("Buscar!")
        // eventBus.writer.onNext(PatientReply(UUID.randomUUID().toString, text, text, text))
        patientRepository.findByName(text, p => eventBus.emit(p))
      }
      ()
    }

    def onKeyPressObserver(input: Input) =
      Observer[dom.KeyboardEvent](onNext = { _ =>
        searchName(input)
        ()
      })

    val _input = input(
      cls := "border-2 border-gray-300 bg-white h-10 px-5 pr-10 rounded-lg text-sm focus:outline-none",
      typ := "search",
      name := "search",
      placeholder := "Nombre del paciente",
      "searchInput",
      autoFocus(true),
      inContext(thisNode => onKeyPress.filter(e => e.keyCode == dom.ext.KeyCode.Enter) --> onKeyPressObserver(thisNode)),
    )

    div(
      cls := "py-6 flex flex-col justify-center sm:py-12",
      div(
        cls := "pt-2 relative mx-auto text-gray-600",
        _input,
        button(
          //          typ := "submit",
          cls := "absolute right-0 top-0 mt-5 mr-4",
          onClick --> Observer[dom.MouseEvent](onNext = { _ =>
            searchName(_input); ()
          }),
          svg.svg(
            svg.cls := "text-gray-600 h-4 w-4 fill-current",
            //            svg.xmlns := "http://www.w3.org/2000/svg",
            //            svg.xlinkHref := "http://www.w3.org/1999/xlink",
            svg.idAttr := "Capa_1",
            svg.x := "0px",
            svg.y := "0px",
            svg.viewBox := "0 0 56.966 56.966",
            svg.style := "enable-background:new 0 0 56.966 56.966;",
            svg.xmlSpace := "preserve",
            svg.width := "512px",
            svg.height := "512px",
            svg.path(
              svg.d := "M55.146,51.887L41.588,37.786c3.486-4.144,5.396-9.358,5.396-14.786c0-12.682-10.318-23-23-23s-23,10.318-23,23  s10.318,23,23,23c4.761,0,9.298-1.436,13.177-4.162l13.661,14.208c0.571,0.593,1.339,0.92,2.162,0.92  c0.779,0,1.518-0.297,2.079-0.837C56.255,54.982,56.293,53.08,55.146,51.887z M23.984,6c9.374,0,17,7.626,17,17s-7.626,17-17,17  s-17-7.626-17-17S14.61,6,23.984,6z"
            ),
          ),
        ),
      ),
    )
  }
  /*
      <div class="py-6 flex flex-col justify-center sm:py-12">
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

  private def searchButton(
      eventBus: EventBus[PatientReply],
      input: ReactiveHtmlElement[html.Input],
  ): ReactiveHtmlElement[html.Button] = {

    val onClickObserver = Observer[dom.MouseEvent](onNext = { event =>
      debug(s"mouseEvent: $event")
      val text = input.ref.value
      // if (text.trim.nonEmpty) {
      debug(s"Buscar! $text")
      // eventBus.emit(PatientReply("-3", text))
      // }
    })

    button("buscar", cls := "btn btn-outline-primary", onClick --> onClickObserver)
  }

  private def searchTable(eventBus: EventBus[PatientReply], commandWriteBus: WriteBus[Command]): HtmlElement = {
    val tds = tbody(children.command <-- eventBus.events.map(p => CollectionCommand.Append(_td(p, commandWriteBus))))
    TableBasic(List("Paciente"), Some(tds))
  }

  private def _td(patientReply: PatientReply, commandWriteBus: WriteBus[Command]): HtmlElement = {

    val showHistoryOnClickObserver = Observer[dom.MouseEvent](onNext = { event =>
      debug(s"event: $event")
      debug(s"1: ${event.target.isInstanceOf[HTMLTableCellElement]}")
      val names = patientReply.name.split(" ").asInstanceOf[Array[String]]
      val patient = new Patient(
        "Test",
        new HumanName(patientReply.paternalSurname, patientReply.maternalSurname, names),
        true,
        LocalDate.of(1979, 10, 13).asInstanceOf[LocalDate], // FIXME
        Seq(new ContactPoint(SystemContactPoint.PHONE, "993990103")),
      )
      commandWriteBus.onNext(ShowPatient(patient))
      ()
    })

    //                    tr(td(idAttr := s"${i._2}", i._1.name, onClick --> obsHistory))
    tr(
      td(
        cls := "px-6 py-4 whitespace-nowrap",
        div(
          cls := "flex items-center",
          div(
            cls := "flex-shrink-0 h-10 w-10",
            img(
              cls := "h-10 w-10 rounded-full",
              src := "https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-1.2.1&amp;ixid=eyJhcHBfaWQiOjEyMDd9&amp;auto=format&amp;fit=facearea&amp;facepad=4&amp;w=256&amp;h=256&amp;q=60",
            ),
          ),
          div(
            cls := "ml-4",
            div(cls := "text-2xl font-medium font-semibold text-gray-900", patientReply.name),
            div(cls := "text-sm text-gray-500", "test"),
          ),
        ),
        onClick --> showHistoryOnClickObserver,
      )
    )
  }
}
