package medical.ui.component.atoms

import com.raquo.laminar.api.L._

object InputLabel {

  def apply(_name: String, description: String, _value: String = null): HtmlElement = {
    div(
      label(forId := _name, description, cls := "mx-2 w-16 inline-block"),
      input(idAttr := "name", name := _name, readOnly := true, value := Option(_value).orNull,
        cls := "mx-2 px-3 py-2 placeholder-gray-300 border border-gray-300 rounded-md focus:outline-none focus:ring focus:ring-indigo-100 focus:border-indigo-300")
    )
  }

}
