import sbt._

object Dependencies extends Akka with Gatling with Grpc {

  val logbackClassic = "ch.qos.logback" % "logback-classic" % "1.2.11"

  val scribeVersion = "3.8.3"
  val scribe = "com.outr" %% "scribe" % scribeVersion
  val scribeSlf4j = "com.outr" %% "scribe-slf4j" % scribeVersion

  lazy val algolia = "3.16.5"
  val algoliaCore = "com.algolia" % "algoliasearch-core" % algolia
  val algoliaApache = "com.algolia" % "algoliasearch-apache" % algolia
  val algoliaScala = "com.algolia" %% "algoliasearch-scala" % "1.45.0"
  val algoliaNet = "com.algolia" % "algoliasearch-java-net" % algolia

  lazy val scalatestVersion = "3.2.12"
  val scalaTest = "org.scalatest" %% "scalatest" % scalatestVersion % Test
}

trait Akka {

  lazy val akkaVersion = "2.6.19"
  lazy val alpakkaKafkaVersion = "3.0.0"
  lazy val akkaHttpVersion = "10.2.9"

  val akkaHttp = "com.typesafe.akka" %% "akka-http" % akkaHttpVersion
  val akkaActorTyped = "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion
  val akkaStream = "com.typesafe.akka" %% "akka-stream" % akkaVersion
  val akkaDiscovery = "com.typesafe.akka" %% "akka-discovery" % akkaVersion
  val akkaPersistenceTyped = "com.typesafe.akka" %% "akka-persistence-typed" % akkaVersion
  val akkaPersistenceDatastore = "de.innfactory" %% "akka-persistence-gcp-datastore" % "1.0.1"
  val akkaStreamKafka = "com.typesafe.akka" %% "akka-stream-kafka" % alpakkaKafkaVersion
  val akkaSlf4j = "com.typesafe.akka" %% "akka-slf4j" % akkaVersion
  // "com.typesafe.akka" %% "akka-pki" % akkaVersion,
  val akkaHttpCors = "ch.megard" %% "akka-http-cors" % "1.1.3" // Para poder usar akka grpc con grpc-web
  val akkaGrpcRuntime = "com.lightbend.akka.grpc" %% "akka-grpc-runtime" % "2.1.4"
  val macwire = "com.softwaremill.macwire" %% "macros" % "2.5.7" % Provided

  //val akkaTestkit = "com.typesafe.akka" %% "akka-testkit" % akkaVersion % Test
  val akkaPersistenceTck = "com.typesafe.akka" %% "akka-persistence-tck" % akkaVersion
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
  val gatlingVersion = "3.7.6"
  val gatlingTest = "io.gatling" % "gatling-test-framework" % gatlingVersion % "test"
  val gatlingChartsTest = "io.gatling.highcharts" % "gatling-charts-highcharts" % gatlingVersion % "test"
  val gatlingGrpcTest = "com.github.phisgr" % "gatling-grpc" % "0.13.0" % "test"
}
