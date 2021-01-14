package medical

import com.raquo.laminar.api.L._

object PatientSection {
  def apply(patientBasic: PatientBasic): HtmlElement = {
    section(
      className := "container",
      div(
        className := "d-flex align-items-center p-3 my-3 bg-purple rounded shadow-sm",
        img(className := "me-3", width := "48px", height := "38px", src := "https://getbootstrap.com/docs/5.0/assets/brand/bootstrap-logo-white.svg"),
        div(
          className := "1h-1",
          h1(className := "h6 mb-0 lh-1", s"Paciente"),
          small(patientBasic.name),
        ),
      ),
      div(
        className := "my-3 p-3 bg-white rounded shadow-sm",
        h6(
          className := "d-flex flex-column flex-md-row align-items-center border-bottom pb-2 mb-0",
          span(className := "me-md-auto", "Historias clinicas"),
          a(className := "btn btn-outline-primary", i(className := "bi bi-plus"), " Agregar historia"),
        ),
        div(
          className := "d-flex text-muted pt-3",
          p(className := "pb-3 mb-0 small lh-sm border-bottom", "Test"),
        ),
        div(
          className := "d-flex text-muted pt-3",
          p(className := "pb-3 mb-0 small lh-sm border-bottom", "Test"),
        ),
        div(
          className := "d-flex text-muted pt-3",
          p(className := "pb-3 mb-0 small lh-sm border-bottom", "Test"),
        ),
      ),
    )
  }
}
