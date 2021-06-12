package medical.infrastructure.ui.atom

import com.raquo.laminar.api.L._

object CloseSvg {
  def apply(): SvgElement = {
    svg.svg(
      svg.cls := "ml-auto fill-current text-gray-700 w-6 h-6 cursor-pointer",
      svg.viewBox := "0 0 18 18",
      svg.path(
        svg.d := "M14.53 4.53l-1.06-1.06L9 7.94 4.53 3.47 3.47 4.53 7.94 9l-4.47 4.47 1.06 1.06L9 10.06l4.47 4.47 1.06-1.06L10.06 9z",
      ),
    )
  }
}
