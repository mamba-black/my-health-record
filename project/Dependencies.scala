import sbt._

object Dependencies extends Akka with Gatling with Grpc {

  lazy val logbackClassic = "ch.qos.logback" % "logback-classic" % "1.2.11"

  lazy val scribeVersion = "3.8.3"
  lazy val scribe = "com.outr" %% "scribe" % scribeVersion
  lazy val scribeSlf4j = "com.outr" %% "scribe-slf4j" % scribeVersion

  lazy val algolia = "3.16.5"
  lazy val algoliaCore = "com.algolia" % "algoliasearch-core" % algolia
  lazy val algoliaApache = "com.algolia" % "algoliasearch-apache" % algolia
  lazy val algoliaScala = "com.algolia" %% "algoliasearch-scala" % "1.45.0"
  lazy val algoliaNet = "com.algolia" % "algoliasearch-java-net" % algolia

  lazy val scalatestVersion = "3.2.12"
  lazy val scalaTest = "org.scalatest" %% "scalatest" % scalatestVersion % Test
}

trait Akka {

  lazy val akkaVersion = "2.6.19"
  lazy val alpakkaKafkaVersion = "3.0.0"
  lazy val akkaHttpVersion = "10.2.9"

  lazy val akkaHttp = "com.typesafe.akka" %% "akka-http" % akkaHttpVersion
  lazy val akkaActorTyped = "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion
  lazy val akkaStream = "com.typesafe.akka" %% "akka-stream" % akkaVersion
  lazy val akkaDiscovery = "com.typesafe.akka" %% "akka-discovery" % akkaVersion
  lazy val akkaPersistenceTyped = "com.typesafe.akka" %% "akka-persistence-typed" % akkaVersion
  lazy val akkaPersistenceDatastore = "de.innfactory" %% "akka-persistence-gcp-datastore" % "1.0.1"
  // lazy val akkaSerializationJackson = "com.typesafe.akka" %% "akka-serialization-jackson" % akkaVersion
  lazy val akkaStreamKafka = "com.typesafe.akka" %% "akka-stream-kafka" % alpakkaKafkaVersion
  lazy val akkaSlf4j = "com.typesafe.akka" %% "akka-slf4j" % akkaVersion
  // "com.typesafe.akka" %% "akka-pki" % akkaVersion,
  lazy val akkaHttpCors = "ch.megard" %% "akka-http-cors" % "1.1.3" // Para poder usar akka grpc con grpc-web
  lazy val akkaGrpcRuntime = "com.lightbend.akka.grpc" %% "akka-grpc-runtime" % "2.1.4"
  lazy val macwire = "com.softwaremill.macwire" %% "macros" % "2.5.7" % Provided

  // val akkaTestkit = "com.typesafe.akka" %% "akka-testkit" % akkaVersion % Test
  lazy val akkaPersistenceTck = "com.typesafe.akka" %% "akka-persistence-tck" % akkaVersion
  lazy val akkaActorTypedTest = "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test
  lazy val akkaStreamTest = "com.typesafe.akka" %% "akka-stream-testkit" % akkaVersion % Test
  lazy val akkaPersistenceTest = "com.typesafe.akka" %% "akka-persistence-testkit" % akkaVersion % Test
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
