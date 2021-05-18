package medical.ui.component.molecule

import com.raquo.domtypes.generic.codecs.StringAsIsCodec
import com.raquo.laminar.api.L.*
import medical.domain.LoginGoogleResponse
import org.scalajs.dom
import scribe.*

import scala.scalajs.js.JSON
import scala.scalajs.js.annotation.JSExportTopLevel

object LoginGoogleOpenIdConnect {
  def apply(): HtmlElement = {
    div(
      div(
        idAttr := "g_id_onload",
        customHtmlAttr("data-client_id", StringAsIsCodec) := "937186309482-uc7dm6bc6o6p3disa546hq25n8dbov42.apps.googleusercontent.com",
        customHtmlAttr("data-login_uri", StringAsIsCodec) := "http://localhost",
        customHtmlAttr("data-callback", StringAsIsCodec) := "handleCredentialResponse",
        //              customHtmlAttr("data-auto_prompt", StringAsIsCodec) := "false",
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
  }
}
