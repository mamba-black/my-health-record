package medical.infraestructure

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.grpc.scaladsl.{ServerReflection, WebHandler}
import akka.http.scaladsl.Http
import akka.http.scaladsl.model.HttpMethods.GET
import akka.http.scaladsl.model.{ContentTypes, HttpEntity, HttpRequest, HttpResponse, Uri}
import com.typesafe.config.ConfigFactory
import grpc.health.v1.HealthHandler
import medical.api.{PatientApi, PatientApiHandler}
import medical.application.PatientServiceImpl
import medical.infraestructure.repository.PatientRepositoryImpl
import medical.presentation.{HealthImpl, PatientApiImpl}

import scala.concurrent.ExecutionContextExecutor
//import wvlet.log.LogFormatter.PlainSourceCodeLogFormatter
import scribe._

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

    new PatientServer(system).run()
    ()
  }
}

class PatientServer(system: ActorSystem[_]) {
  def run(): Future[Http.ServerBinding] = {
    implicit val sys: ActorSystem[_] = system
    implicit val ec: ExecutionContextExecutor = sys.executionContext

    val serverReflection = ServerReflection.partial(List(PatientApi))
    val health = HealthHandler.partial(new HealthImpl())
    val patientApi =
      PatientApiHandler.partial(new PatientApiImpl(system, new PatientServiceImpl(new PatientRepositoryImpl)))
    val apis = WebHandler.grpcWebHandler(health, serverReflection, patientApi)

    val requestHandler: HttpRequest => Future[HttpResponse] = {
      case HttpRequest(GET, Uri.Path("/"), _, _, _) =>
        Future.successful(
          HttpResponse(entity = HttpEntity(ContentTypes.`text/html(UTF-8)`, "<html><body>Hello world!</body></html>"))
        )

      case HttpRequest(GET, Uri.Path("/ping"), _, _, _) =>
        Future.successful(HttpResponse(entity = "PING!"))

      case r: HttpRequest =>
        apis(r)
    }

    val httpExt = Http()
    val bindingFuture = httpExt
      .newServerAt(interface = "0.0.0.0", port = 9090)
      //.enableHttps(serverHttpContext)
      .bind(requestHandler)
      .map(_.addToCoordinatedShutdown(hardTerminationDeadline = 10.seconds))

    bindingFuture.onComplete {
      case Success(binding) =>
        val address = binding.localAddress
        system.log.info("Success")
        info(s"...gRPC server bound to ${address.getHostString}:${address.getPort}")
      case Failure(exception) =>
        system.log.info("Failure")
        error("Failed to bind gRPC endpoint, terminating system", exception)
        system.terminate()
    }

    bindingFuture
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
