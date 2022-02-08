package computerdatabase

// stringToExpression is hidden because we have $ in GrpcDsl
//import io.gatling.core.Predef.{stringToExpression => _, _}

import com.github.phisgr.gatling.grpc.Predef.{ grpc, * }
import com.github.phisgr.gatling.grpc.protocol.StaticGrpcProtocol
import com.github.phisgr.gatling.grpc.request.ClientStream
import io.gatling.core.Predef.*
import io.gatling.core.structure.ScenarioBuilder

import scala.concurrent.duration.DurationInt

//import scala.concurrent.duration._

class BasicSimulation extends Simulation {

  val grpcProtocol: StaticGrpcProtocol = grpc(managedChannelBuilder("localhost", 9090).usePlaintext())
  // .warmUpCall(PatientApiGrpc.METHOD_GET, PatientIdRequest.defaultInstance)

  val clientStream: ClientStream = grpc("Stream")
    .clientStream("call")

  val scn: ScenarioBuilder = scenario("Scenario Name") // A scenario is a chain of requests and pauses
    .exec(
      grpc("Get")
        .rpc(PatientApiGrpc.METHOD_GET)
        .payload(PatientIdRequest("test"))
    )
    .exec(
      grpc("Get")
        .rpc(PatientApiGrpc.METHOD_GET)
        .payload(PatientIdRequest("test"))
    )

  setUp(
    // scn.inject(atOnceUsers(10))
    scn.inject(constantUsersPerSec(10).during(5.seconds))
  ).protocols(grpcProtocol)
}
