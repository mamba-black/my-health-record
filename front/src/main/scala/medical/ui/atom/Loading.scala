package medical.ui.atom

import com.raquo.laminar.api.L._

object Loading {

  def apply(): HtmlElement = {
    div(
//      img(
//        src := "https://ignitecxmp.com/Images/Newloading.gif",
//        cls := "",
//      ),
      span(
        cls := List("text-green-500", "opacity-75", "my-0", "mx-auto", "block", "relative", "w-0", "h-0"),
        styleAttr := "top: 50%",
        svg.svg(
          svg.cls := "animate-spin -ml-1 mr-3 h-5 w-5 text-white",
          svg.viewBox := "0 0 24 24",
          svg.circle(svg.cls := "opacity-25", svg.cx := "12", svg.cy := "12", svg.r := "10", svg.stroke := "currentColor", svg.strokeWidth := "4"),
          svg.path(svg.cls := "opacity-75", svg.fill("currentColor"), svg.d := "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"),
        ),
      ),
    )
  }

}

/*
<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>

<div class="w-full h-full fixed block top-0 left-0 bg-white opacity-75 z-50">
  <span class="text-green-500 opacity-75 top-1/2 my-0 mx-auto block relative w-0 h-0" style="
    top: 50%;
">
    <i class="fas fa-circle-notch fa-spin fa-5x"></i>
  </span>
</div>
*/
