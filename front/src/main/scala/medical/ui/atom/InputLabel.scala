package medical.ui.atom

import com.raquo.laminar.api.L.*
import scribe.*

object InputLabel {
  def apply(
      _name: String,
      description: String,
      newValue: Signal[Option[String]] = null,
      readOnlySignal: StrictSignal[Boolean],
      inputType: Option[String] = None,
      important: Boolean = false,
  ): HtmlElement = {
    val isLoaded: Signal[Boolean] = newValue.map(v => v.isDefined)
    debug(s"readOnlySignal: $readOnlySignal")

    val inputCss = List(
      "px-3",
      " py-2",
      "w-full",
      "border",
      "border-gray-300",
      "rounded-md",
      "focus:outline-none",
      "focus:ring",
      "block",
    ) ::: (if (important) List("placeholder-red-600", "focus:ring-red-200", "focus:border-red-300", "text-red-900")
           else List("placeholder-gray-300", "focus:ring-indigo-100", "focus:border-indigo-300"))

    val labelCss =
      List("mb-2", "block", "tracking-wide", "uppercase", "font-bold", "text-gray-600", "text-xs") ::: (if (important)
                                                                                                          List(
                                                                                                            "text-red-900"
                                                                                                          )
                                                                                                        else Nil)

    div(
      span(
        cls := ("animate-pulse", "bg-gray-200", "h-5", "w-40", "mb-2", "block", "tracking-wide", "rounded-md"),
        cls.toggle("hidden") <-- isLoaded,
      ),
      span(
        cls := ("animate-pulse", "bg-gray-200", "px-3", " py-2", "h-10", "w-full", "placeholder-gray-300", "rounded-md", "block"),
        cls.toggle("hidden") <-- isLoaded,
      ),
      label(forId := _name, description, cls := labelCss, cls.toggle("hidden") <-- isLoaded.map(!_)),
      input(
        idAttr := "name",
        name := _name,
        //autoComplete := "off",
        `type` := inputType.orNull,
        readOnly <-- readOnlySignal,
        value <-- newValue.map(_.orNull),
        defaultValue <-- newValue.map(_.orNull),
        cls := inputCss,
        cls.toggle("bg-gray-50") <-- readOnlySignal,
        cls.toggle("hidden") <-- isLoaded.map(!_),
      ),
    )
  }

}
