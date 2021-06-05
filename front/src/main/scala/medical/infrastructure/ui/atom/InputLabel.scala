package medical.infrastructure.ui.atom

import com.raquo.laminar.api.L._

object InputLabel {
  def apply(_name: String, description: String, _value: String = null, readOnlySignal: StrictSignal[Boolean], inputType: Option[String] = None): HtmlElement = {
    div(
      label(forId := _name, description, cls := ("mb-2", "block", "tracking-wide", "uppercase", "font-bold", "text-gray-600", "text-xs")),
      input(
        idAttr := "name",
        name := _name,
        `type` := inputType.orNull,
        readOnly <-- readOnlySignal,
        value := Option(_value).orNull,
        cls := ("px-3", " py-2", "w-full", "placeholder-gray-300", "border", "border-gray-300", "rounded-md", "focus:outline-none", "focus:ring", "focus:ring-indigo-100", "focus:border-indigo-300", "block"),
        cls.toggle("bg-gray-50") <-- readOnlySignal,
      )
    )
  }

}
