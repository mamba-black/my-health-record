package medical.infrastructure.ui.atom

import com.raquo.laminar.api.L._

object InputLabel {
  def apply(_name: String, description: String, _value: Signal[Option[String]] = null, readOnlySignal: StrictSignal[Boolean], inputType: Option[String] = None): HtmlElement = {
    val isLoaded: Signal[Boolean] = _value.map(v => v.isDefined)
    div(
      span(
        cls := ("animate-pulse", "bg-gray-200", "h-5", "w-40", "mb-2", "block", "tracking-wide", "rounded-md"),
        cls.toggle("hidden") <-- isLoaded,
      ),
      span(
        cls := ("animate-pulse", "bg-gray-200", "px-3", " py-2", "h-10", "w-full", "placeholder-gray-300", "rounded-md", "block"),
        cls.toggle("hidden") <-- isLoaded,
      ),
      label(
        forId := _name, description,
        cls := ("mb-2", "block", "tracking-wide", "uppercase", "font-bold", "text-gray-600", "text-xs"),
        cls.toggle("hidden") <-- isLoaded.map(!_),
      ),
      input(
        idAttr := "name",
        name := _name,
        `type` := inputType.orNull,
        readOnly <-- readOnlySignal,
        value <-- _value.map(_.orNull),
        defaultValue <-- _value.map(_.orNull),
        cls := ("px-3", " py-2", "w-full", "placeholder-gray-300", "border", "border-gray-300", "rounded-md", "focus:outline-none", "focus:ring", "focus:ring-indigo-100", "focus:border-indigo-300", "block"),
        cls.toggle("bg-gray-50") <-- readOnlySignal,
        cls.toggle("hidden") <-- isLoaded.map(!_),
      ),
    )
  }

}
