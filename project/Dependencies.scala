import sbt._

object Dependencies extends Akka with Gatling with Grpc {

  val logbackClassic = "ch.qos.logback" % "logback-classic" % "1.2.5"
  val scribe = "com.outr" %% "scribe" % "3.5.5"

  //lazy val algolia = "3.14.1"
  //"com.algolia" % "algoliasearch-core" % algolia,
  //"com.algolia" % "algoliasearch-java-net" % algolia,

  lazy val scalatestVersion = "3.2.9"
  val scalaTest = "org.scalatest" %% "scalatest" % scalatestVersion % Test
}

trait Akka {

  lazy val akkaVersion = "2.6.16"
  lazy val alpakkaKafkaVersion = "2.1.1"
  lazy val akkaHttpVersion = "10.2.6"

  val akkaHttp = "com.typesafe.akka" %% "akka-http" % akkaHttpVersion
  val akkaActorTyped = "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion
  val akkaStream = "com.typesafe.akka" %% "akka-stream" % akkaVersion
  val akkaDiscovery = "com.typesafe.akka" %% "akka-discovery" % akkaVersion
  val akkaPersistenceTyped = "com.typesafe.akka" %% "akka-persistence-typed" % akkaVersion
  val akkaStreamKafka = "com.typesafe.akka" %% "akka-stream-kafka" % alpakkaKafkaVersion
  val akkaSlf4j = "com.typesafe.akka" %% "akka-slf4j" % akkaVersion
  //"com.typesafe.akka" %% "akka-pki" % akkaVersion,
  val akkaHttpCors = "ch.megard" %% "akka-http-cors" % "1.1.2" // Para poder usar akka grpc con grpc-web
  val akkaGrpcRuntime = "com.lightbend.akka.grpc" %% "akka-grpc-runtime" % "2.0.0"

  val akkaActorTypedTest = "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test
  val akkaStreamTest = "com.typesafe.akka" %% "akka-stream-testkit" % akkaVersion % Test
  val akkaPersistenceTest = "com.typesafe.akka" %% "akka-persistence-testkit" % akkaVersion % Test
}

trait Grpc {
  val scalapbNetty = "io.grpc" % "grpc-netty" % scalapb.compiler.Version.grpcJavaVersion
  val scalapbRuntime =
    "com.thesamet.scalapb" %% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf"
  val scalapbRuntimeGrpc = "com.thesamet.scalapb" %% "scalapb-runtime-grpc" % scalapb.compiler.Version.scalapbVersion
}

trait Gatling {
  val gatlingVersion = "3.6.1"
  val gatlingTest = "io.gatling" % "gatling-test-framework" % gatlingVersion % "test"
  val gatlingChartsTest = "io.gatling.highcharts" % "gatling-charts-highcharts" % gatlingVersion % "test"
  val gatlingGrpcTest = "com.github.phisgr" % "gatling-grpc" % "0.11.1" % "test"
}
