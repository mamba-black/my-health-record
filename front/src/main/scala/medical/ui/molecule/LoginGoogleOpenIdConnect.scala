package medical.ui.molecule

import com.raquo.domtypes.generic.codecs.StringAsIsCodec
import com.raquo.laminar.api.L.*
import medical.ui.MainUI
import org.scalajs.dom
import scribe.*

import scala.scalajs.js
import scala.scalajs.js.JSON
import scala.scalajs.js.annotation.JSExportTopLevel

object LoginGoogleOpenIdConnect {
  val clientId = "937186309482-uc7dm6bc6o6p3disa546hq25n8dbov42.apps.googleusercontent.com"
  val loginId = "http://localhost:8080"

  def apply(): HtmlElement = {
    div(
      script(
        src := "https://accounts.google.com/gsi/client",
        customHtmlAttr("async", StringAsIsCodec) := "",
        customHtmlAttr("defer", StringAsIsCodec) := "",
      ),
      div(
        idAttr := "g_id_onload",
        customHtmlAttr("data-client_id", StringAsIsCodec) := clientId,
        //customHtmlAttr("data-login_uri", StringAsIsCodec) := loginId,
        customHtmlAttr("data-callback", StringAsIsCodec) := "handleCredentialResponse",
        customHtmlAttr("data-auto_prompt", StringAsIsCodec) := "true",
      ),
      div(
        idAttr := "g_id_signin",
        customHtmlAttr("data-type", StringAsIsCodec) := "standard",
        customHtmlAttr("data-size", StringAsIsCodec) := "large",
        customHtmlAttr("data-theme", StringAsIsCodec) := "outline",
        customHtmlAttr("data-text", StringAsIsCodec) := "sign_in_with",
        customHtmlAttr("data-shape", StringAsIsCodec) := "rectangular",
        customHtmlAttr("data-logo_alignment", StringAsIsCodec) := "left",
      ),
    )
  }

  @JSExportTopLevel("handleCredentialResponse")
  def handleCredentialResponse(response: LoginGoogleResponse): Unit = {
    debug(s"response: ${JSON.stringify(response)}")
    dom.window.localStorage.setItem("session", response.credential)
    render(dom.document.body, MainUI())
  }
}

class LoginGoogleResponse(val clientId: String, val credential: String, val select_by: String) extends js.Object
