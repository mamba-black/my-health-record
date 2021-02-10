package medical

import akka.actor.typed.ActorSystem
import akka.actor.typed.scaladsl.Behaviors
import akka.grpc.scaladsl.{ ServerReflection, WebHandler }
import akka.http.scaladsl.Http
import com.typesafe.config.ConfigFactory
import medical.backend.{ PatientService, PatientServiceHandler }
import wvlet.log.LogFormatter.PlainSourceCodeLogFormatter
import wvlet.log.LogSupport

import scala.concurrent.Future
import scala.concurrent.duration.DurationInt
import scala.util.{ Failure, Success }

object PatientServer extends LogSupport {
  wvlet.log.Logger.setDefaultFormatter(PlainSourceCodeLogFormatter)
  def main(args: Array[String]): Unit = {
    info("Starting gRPC...")
    val conf = ConfigFactory.parseString("akka.http.server.preview.enable-http2 = on")
      .withFallback(ConfigFactory.defaultApplication())
    val system = ActorSystem[Nothing](Behaviors.empty, "PatientServer", conf)
    new PatientServer(system).run()
  }
}

class PatientServer(system: ActorSystem[_]) extends LogSupport {
  def run(): Future[Http.ServerBinding] = {
    implicit val sys = system
    implicit val ec = sys.executionContext

    val patientService = PatientServiceHandler.partial(new PatientServiceImpl(system))
    val serverReflection = ServerReflection.partial(List(PatientService))
    val services = WebHandler.grpcWebHandler(serverReflection, patientService)

    val bindingFuture = Http(system)
      .newServerAt(interface = "0.0.0.0", port = 8080)
//      .enableHttps(serverHttpContext)
      .bind(services)
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
