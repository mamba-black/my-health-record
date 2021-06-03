package medical.infrastructure.ui.atom

import com.raquo.laminar.api.L._

object Loading {

  def apply(): HtmlElement = {
    div(
      img(
        src := "https://ignitecxmp.com/Images/Newloading.gif",
        cls := "",
      )
    )
  }

}
