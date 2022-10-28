package medical.infraestructure

import medical.infraestructure.di.PatientModule
import scribe.*

import scala.concurrent.Future
import scala.io.StdIn

object Main {

  def main(args: Array[String]): Unit = {
    info("Starting gRPC...")

    val module = new PatientModule {}
    // AkkaManagement(module.system).start()
    // ClusterBootstrap(module.system).start()

    import module.system.executionContext

    val bindingFuture = Future.sequence(module.patientServer.run())

    StdIn.readLine()
    bindingFuture
      .flatMap(binding => Future.sequence(binding.map(_.unbind())))
      .onComplete(_ => {
        module.system.terminate()
      })
    ()
  }
}
