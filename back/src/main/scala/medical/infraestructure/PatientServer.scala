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
