package medical.infraestructure

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.grpc.scaladsl.{ServerReflection, WebHandler}
import akka.http.scaladsl.{ConnectionContext, Http}
import akka.http.scaladsl.model.*
import akka.http.scaladsl.model.HttpMethods.*
import akka.http.scaladsl.model.headers.{
  `Access-Control-Allow-Headers`,
  `Access-Control-Allow-Methods`,
  `Access-Control-Allow-Origin`,
}
import com.typesafe.config.ConfigFactory
import grpc.health.v1.HealthHandler
import medical.api.{PatientApi, PatientApiHandler}
import medical.application.PatientServiceImpl
import medical.infraestructure.presentation.{HealthImpl, PatientApiImpl}
import medical.infraestructure.repository.PatientRepositoryImpl

import java.security.{KeyStore, SecureRandom}
import javax.net.ssl.{KeyManagerFactory, SSLContext, TrustManagerFactory}
import scala.concurrent.ExecutionContextExecutor
import scala.io.StdIn
//import wvlet.log.LogFormatter.PlainSourceCodeLogFormatter
import scribe.*

import scala.concurrent.Future
import scala.concurrent.duration.DurationInt
import scala.util.{Failure, Success}

object PatientServer {
  //  wvlet.log.Logger.setDefaultFormatter(PlainSourceCodeLogFormatter)
  def main(args: Array[String]): Unit = {
    info("Starting gRPC...")

    val conf = ConfigFactory
      .parseString("akka.http.server.preview.enable-http2 = on")
      .withFallback(ConfigFactory.defaultApplication())
    val system = ActorSystem[Nothing](Behaviors.empty, "PatientServer", conf)
    implicit val executionContext: ExecutionContextExecutor = system.executionContext

    val bindingFuture = Future.sequence(new PatientServer(system).run())

    StdIn.readLine()
    bindingFuture
      .flatMap(binding => Future.sequence(binding.map(_.unbind())))
      .onComplete(_ => {
        system.terminate()
      })
    ()
  }
}

class PatientServer(system: ActorSystem[?]) {
  def run(): List[Future[Http.ServerBinding]] = {
    implicit val sys: ActorSystem[?] = system
    implicit val ec: ExecutionContextExecutor = sys.executionContext

    val serverReflection = ServerReflection.partial(List(PatientApi))
    val health = HealthHandler.partial(new HealthImpl())
    val patientApi =
      PatientApiHandler.partial(new PatientApiImpl(system, new PatientServiceImpl(new PatientRepositoryImpl)))
    val apis = WebHandler.grpcWebHandler(health, serverReflection, patientApi)

    val requestHandler: HttpRequest => Future[HttpResponse] = { request =>
      info("-----------------------------------------------------------------------------------------------")
      info(s"request: ${request.protocol.value} ${request.method.value}")

      request match {
        case HttpRequest(GET, Uri.Path("/"), _, _, _) =>
          Future.successful(
            HttpResponse(entity = HttpEntity(ContentTypes.`text/html(UTF-8)`, "<html><body>Hello world!</body></html>"))
          )

        case HttpRequest(GET, Uri.Path("/ping"), _, _, _) =>
          Future.successful(HttpResponse(entity = "PING!"))

        case HttpRequest(OPTIONS, _, _, _, _) =>
          Future.successful(
            HttpResponse(headers =
              Seq(
                `Access-Control-Allow-Origin`.*,
                `Access-Control-Allow-Headers`(
                  "keep-alive",
                  "user-agent",
                  "cache-control",
                  "content-type",
                  "content-transfer-encoding",
                  "custom-header-1",
                  "x-accept-content-transfer-encoding",
                  "x-accept-response-streaming",
                  "x-user-agent",
                  "x-grpc-web",
                  "grpc-timeout",
                ),
                `Access-Control-Allow-Methods`(POST), //, GET, PUT, DELETE, OPTIONS
              )
            )
          )

        case r: HttpRequest =>
          apis(r)
      }
    }

    // ============================================================================================================
    val httpBinding = Http()
      .newServerAt(interface = "0.0.0.0", port = 9090)
      .bind(requestHandler)
      .map(_.addToCoordinatedShutdown(hardTerminationDeadline = 10.seconds))

    httpBinding.onComplete {
      case Success(binding) =>
        val address = binding.localAddress
        system.log.info("Success")
        info(s"...gRPC server bound to ${address.getHostString}:${address.getPort}")
      case Failure(exception) =>
        system.log.info("Failure")
        error("Failed to bind gRPC endpoint, terminating system", exception)
        system.terminate()
    }
    // ============================================================================================================

    // ============================================================================================================
    /**
      * https://lightbend.github.io/ssl-config/CertificateGeneration.html
      *
      * = AKKA
      *
      * {{{
      *  export PW=Miuler123
      *
      *  ----------------------------------------------------------------------------------------------------------------
      *  # 1. Cree un certificado de CA raíz de par de claves autofirmado.
      *  keytool -genkeypair -v -alias miulerca -dname "CN=miulerca, OU=miuler com, O=Miuler SA, L=Lima, ST=Lima, C=PE" -keystore miulerca.jks -keypass:env PW -storepass:env PW -keyalg RSA -keysize 4096 -ext KeyUsage:critical="KeyCertSign" -ext BasicConstraints:critical="ca:true" -validity 9999
      *
      *  ls
      *  #miulerca.jks
      *
      *  # 2. Exporte el certificado público miulerca como miulerca.crt para que pueda usarse en almacenes de confianza.
      *  keytool -export -v -alias miulerca -file miulerca.crt -keypass:env PW -storepass:env PW -keystore miulerca.jks -rfc
      *
      *  ls
      *  #miulerca.jks
      *  #miulerca.crt
      *
      *
      *  ----------------------------------------------------------------------------------------------------------------
      *  # 1. Cree un certificado de servidor, vinculado a miuler.com
      *  keytool -genkeypair -v -alias miuler.com -dname "CN=miuler.com, OU=miuler com, O=Miuler SA, L=Lima, ST=Lima, C=PE" -keystore miuler.com.jks -keypass:env PW -storepass:env PW -keyalg RSA -keysize 2048 -validity 385
      *
      *  ls
      *  #miulerca.jks
      *  #miulerca.crt
      *  #miuler.com.jks
      *
      *  # 2. Cree una SOLICITUD (request/csr) de firma de certificado para miuler.com
      *  keytool -certreq -v -alias miuler.com -keypass:env PW -storepass:env PW -keystore miuler.com.jks -file miuler.com.csr
      *
      *  ls
      *  #miulerca.jks
      *  #miulerca.crt
      *  #miuler.com.jks
      *  #miuler.com.csr
      *
      *  # 3. Dile a miulerca que firme el certificado de miuler.com. Tenga en cuenta que la extensión está en la solicitud, no en el certificado original.
      *  #    Técnicamente, keyUsage debe ser digitalSignature para DHE o ECDHE, keyEncipherment para RSA.
      *  keytool -gencert -v -alias miulerca -keypass:env PW -storepass:env PW -keystore miulerca.jks -infile miuler.com.csr -outfile miuler.com.crt -ext KeyUsage:critical="digitalSignature,keyEncipherment" -ext EKU="serverAuth" -ext SAN="DNS:miuler.com" -rfc
      *
      *  ls
      *  #miulerca.jks
      *  #miulerca.crt
      *  #miuler.com.jks
      *  #miuler.com.csr
      *  #miuler.com.crt
      *
      *  # 4. Dígale a miuler.com.jks que puede confiar en miulerca como firmante.
      *  keytool -import -v -alias miulerca -file miulerca.crt -keystore miuler.com.jks -storetype JKS -storepass:env PW
      *  # Confirmar
      *
      *  # 5. Importar el certificado firmado en miuler.com.jks
      *  keytool -import -v -alias miuler.com -file miuler.com.crt -keystore miuler.com.jks -storetype JKS -storepass:env PW
      *
      *
      *  ----------------------------------------------------------------------------------------------------------------
      *  # 6. Enumere el contenido de example.com.jks solo para confirmarlo.
      *  #    Si está utilizando Play como un punto de terminación TLS, este es el almacén de claves que debe presentar como servidor
      *  keytool -list -v -keystore miuler.com.jks -storepass:env PW
      *
      *  # 7. Solol para probar listar dentro de miulerca.jks
      *  keytool -list -v -keystore miulerca.jks -storepass:env PW
      *
      *  # 8. Mostrar el certificado en AKKA HTTP
      *  keytool -printcert -sslserver localhost:9443 -rfc
      * }}}
      *
      *
      * = Nginx
      *
      * {{{
      *  ----------------------------------------------------------------------------------------------------------------
      *  # 1. NGINX solo trabaja con los certificados PEM, no trabaja con el formato que java entiende, pero keytool tampoco
      *  #    puede generar certificados de tipo PEM, por lo que hay que hacer un paso intermedio usando openssl:
      *  keytool -importkeystore -v -srcalias miuler.com -srckeystore miuler.com.jks -srcstoretype jks -srcstorepass:env PW -destkeystore miuler.com.p12 -destkeypass:env PW -deststorepass:env PW -deststoretype PKCS12
      *
      *  # 2. Exporte la clave privada de miuler.com para usarla en nginx. Tenga en cuenta que esto requiere el uso de OpenSSL
      *  openssl pkcs12 -nocerts -nodes -passout env:PW -passin env:PW -in miuler.com.p12 -out miuler.com.key
      *
      *  ls
      *  #miulerca.jks
      *  #miulerca.crt
      *  #miuler.com.jks
      *  #miuler.com.csr
      *  #miuler.com.crt
      *  #miuler.com.key
      *
      *  # 3. Mostrar los certificados del nginx
      *  keytool -printcert -sslserver localhost:443 -rfc
      * }}}
      *
      * Cargar los certificados en el nginx, nginx.conf:
      *
      * {{{
      * ssl_certificate     /etc/nginx/certs/miuler.com.crt
      * ssl_certificate_key /etc/nginx/certs/miuler.com.key
      * }}}
      *
      *
      * = Cliente
      *
      * Hay dos partes para configurar un cliente: configurar un almacén de confianza y configurar la autenticación del cliente.
      * Configuración de una tienda de confianza
      *
      * Todos los clientes deben ver que el certificado example.com del servidor es de confianza, pero no es necesario
      *
      * que vean la clave privada. Genere un almacén de confianza que contenga solo el certificado
      * y entréguelo a los clientes. Muchos clientes de Java prefieren tener el almacén de confianza en formato JKS.
      *
      * {{{
      *  ----------------------------------------------------------------------------------------------------------------
      *  # 1. Crear el storage para el cliente
      *  keytool -import -v -alias miulerca -file miulerca.crt -keypass:env PW -storepass clientepass -keystore miulerca.trust.jks
      *
      *  # 2. Listar el certificado que esta en el store del cliente
      *  keytool -list -v -keystore miulerca.trust.jks -storepass clientepass
      *
      *  # 3. Ejemplo de TrustManager
      *  ssl-config {
      *    trustManager = {
      *      stores = [
      *        { path = "/Users/wsargent/work/ssltest/conf/miulerca.trust.jks" }
      *      ]
      *    }
      *  }
      * }}}
      *
      *
      *
      * {{{
      * keytool -genkeypair -v \
      *   -alias clientca \
      *   -keystore client.jks \
      *   -dname "CN=clientca, OU=miuler com, O=Miuler SA, L=Lima, ST=Lima, C=PE" \
      *   -keypass:env PW \
      *   -storepass:env PW \
      *   -keyalg RSA \
      *   -keysize 4096 \
      *   -ext KeyUsage:critical="keyCertSign" \
      *   -ext BasicConstraints:critical="ca:true" \
      *   -validity 9999
      *
      * # Create another key pair that will act as the client.
      * keytool -genkeypair -v \
      *   -alias client \
      *   -keystore client.jks \
      *   -dname "CN=client, OU=miuler com, O=Miuler SA, L=Lima, ST=Lima, C=PE" \
      *   -keypass:env PW \
      *   -storepass:env PW \
      *   -keyalg RSA \
      *   -keysize 2048
      *
      * # Create a certificate signing request from the client certificate.
      * keytool -certreq -v \
      *   -alias client \
      *   -keypass:env PW \
      *   -storepass:env PW \
      *   -keystore client.jks \
      *   -file client.csr
      *
      * # Make clientCA create a certificate chain saying that client is signed by clientCA.
      * keytool -gencert -v \
      *   -alias clientca \
      *   -keypass:env PW \
      *   -storepass:env PW \
      *   -keystore client.jks \
      *   -infile client.csr \
      *   -outfile client.crt \
      *   -ext EKU="clientAuth" \
      *   -rfc
      *
      * # Export the client-ca certificate from the keystore.  This goes to nginx under "ssl_client_certificate"
      * # and is presented in the CertificateRequest.
      * keytool -export -v \
      *   -alias clientca \
      *   -file clientca.crt \
      *   -storepass:env PW \
      *   -keystore client.jks \
      *   -rfc
      *
      * # Import the signed certificate back into client.jks.  This is important, as JSSE won't send a client
      * # certificate if it can't find one signed by the client-ca presented in the CertificateRequest.
      * keytool -import -v \
      *   -alias client \
      *   -file client.crt \
      *   -keystore client.jks \
      *   -storetype JKS \
      *   -storepass:env PW
      *
      * # Export the client CA's certificate and private key to pkcs12, so it's safe.
      * keytool -importkeystore -v \
      *   -srcalias clientca \
      *   -srckeystore client.jks \
      *   -srcstorepass:env PW \
      *   -destkeystore clientca.p12 \
      *   -deststorepass:env PW \
      *   -deststoretype PKCS12
      *
      * # Import the client CA's public certificate into a JKS store for Play Server to read.  We don't use
      * # the PKCS12 because it's got the CA private key and we don't want that.
      * keytool -import -v \
      *   -alias clientca \
      *   -file clientca.crt \
      *   -keystore clientca.jks \
      *   -storepass:env PW << EOF
      * yes
      * EOF
      *
      * # Then, strip out the client CA alias from client.jks, just leaving the signed certificate.
      * keytool -delete -v \
      *  -alias clientca \
      *  -storepass:env PW \
      *  -keystore client.jks
      *
      * # List out the contents of client.jks just to confirm it.
      * keytool -list -v \
      *   -keystore client.jks \
      *   -storepass:env PW
      * }}}
      *
      *
      * # Tools
      * xca
      * keystore-explorer-bin
      */
    val password = "Miuler123"
    val keyStoreInputStream = getClass.getClassLoader.getResourceAsStream("certs/miuler.com.jks")
    info(s"keyStoreIputStream: ${keyStoreInputStream}")

    val keyStore = KeyStore.getInstance("PKCS12")
    keyStore.load(keyStoreInputStream, password.toCharArray)
    info(s"keyStore.size(): ${keyStore.size()}")

    val keyManagerFactory = KeyManagerFactory.getInstance("SunX509")
    keyManagerFactory.init(keyStore, password.toCharArray)

    val trustManagerFactory = TrustManagerFactory.getInstance("SunX509")
    trustManagerFactory.init(keyStore)

    val sslContext = SSLContext.getInstance("TLS")
    sslContext.init(keyManagerFactory.getKeyManagers, trustManagerFactory.getTrustManagers, new SecureRandom)

    val httpsConnectionContext = ConnectionContext.httpsServer(sslContext)
    val httpsBinding = Http()
      .newServerAt(interface = "0.0.0.0", port = 9443)
      .enableHttps(httpsConnectionContext)
      .bind(requestHandler)
      .map(_.addToCoordinatedShutdown(hardTerminationDeadline = 10.seconds))

    httpsBinding.onComplete {
      case Success(binding) =>
        val address = binding.localAddress
        system.log.info("Success")
        info(s"...gRPC server bound to ${address.getHostString}:${address.getPort}")
      case Failure(exception) =>
        system.log.info("Failure")
        error("Failed to bind gRPC endpoint, terminating system", exception)
        system.terminate()
    }
    // ============================================================================================================

    List(httpBinding, httpsBinding)
  }

  //  private def serverHttpContext: HttpsConnectionContext = {
  //    val privateKey =
  //      DERPrivateKeyLoader.load(PEMDecoder.decode(readPrivateKeyPem()))
  //    val fact = CertificateFactory.getInstance("X.509")
  //    val cer = fact.generateCertificate(
  //      classOf[PatientServer].getResourceAsStream("/certs/server1.pem")
  //    )
  //    val ks = KeyStore.getInstance("PKCS12")
  //    ks.load(null)
  //    ks.setKeyEntry(
  //      "private",
  //      privateKey,
  //      new Array[Char](0),
  //      Array[Certificate](cer)
  //    )
  //    val keyManagerFactory = KeyManagerFactory.getInstance("SunX509")
  //    keyManagerFactory.init(ks, null)
  //    val context = SSLContext.getInstance("TLS")
  //    context.init(keyManagerFactory.getKeyManagers, null, new SecureRandom)
  //    ConnectionContext.https(context)
  //  }
  //  private def readPrivateKeyPem(): String =
  //    Source.fromResource("certs/server1.key").mkString
}
