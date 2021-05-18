import org.scalajs.sbtplugin.ScalaJSPlugin.autoImport.scalaJSUseMainModuleInitializer
import scalajsbundler.Npm

resolvers += Resolver.sonatypeRepo("public")
ThisBuild / organization := "miuler"
//ThisBuild / libraryDependencies ++= Seq(
//  compilerPlugin("com.github.ghik" % "silencer-plugin" % "1.7.4" cross CrossVersion.full),
//  "com.github.ghik" % "silencer-lib" % "1.7.4" % Provided cross CrossVersion.full
//)
ThisBuild / scalacOptions ++= Seq(
//  "-P:silencer:pathFilters=.*[/]src_managed[/].*",
  //  "-Wconf:src=src_managed/.*:silent",
)

val scala2Version = "2.13.5"
val scala3Version = "3.0.0"

//lazy val dtos = project
//  .in(file("dtos"))
//  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
//  .settings(
//    name := "my-health-record.dtos",
//    scalaVersion := scala2Version,
//    scalaJSUseMainModuleInitializer := true,
//  )

//lazy val akkaVersion = "2.6.14"
//lazy val akkaHttpVersion = "10.2.4"
//lazy val algolia = "3.14.1"
lazy val scalatestVersion = "3.2.9"

//lazy val back = (project in file("back"))
//  .enablePlugins(AkkaGrpcPlugin, JavaAppPackaging, DockerPlugin)
//  .settings(
//    name := "my-health-record.back",
//    scalaVersion := scala2Version,
//    //    Docker / packageName := "gcr.io/miuler-medical-001",
//    //    dockerBaseImage := "ghcr.io/graalvm/graalvm-ce:latest", // Tenemos problemas para detectar el maximo de memoria
//    dockerBaseImage := "adoptopenjdk:11-jre-hotspot",
//    dockerRepository := Some("gcr.io/miuler-medical-001"),
//    dockerExposedPorts := Seq(8080),
//    ThisBuild / dynverSeparator := "-",
//    dockerUpdateLatest := true,
//    //    scalaVersion := "3.0.0-M3",
//    libraryDependencies += "org.wvlet.airframe" %% "airframe-log" % "21.4.1",
//    libraryDependencies ++= Seq(
//      "com.typesafe.akka" %% "akka-http" % akkaHttpVersion,
//      "com.typesafe.akka" %% "akka-http2-support" % akkaHttpVersion,
//      "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion,
//      "com.typesafe.akka" %% "akka-stream" % akkaVersion,
//      "com.typesafe.akka" %% "akka-discovery" % akkaVersion,
//      //      "com.typesafe.akka" %% "akka-pki" % akkaVersion,
//      "ch.megard" %% "akka-http-cors" % "1.1.1", // Para poder usar akka grpc con grpc-web
//      "com.lightbend.akka.grpc" %% "akka-grpc-runtime" % "1.1.1",
//      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
//      "ch.qos.logback" % "logback-classic" % "1.2.3",
//
//      "com.algolia" % "algoliasearch-core" % algolia,
//      "com.algolia" % "algoliasearch-java-net" % algolia,
//
//      //      // The Akka HTTP overwrites are required because Akka-gRPC depends on 10.1.x
//      //      "com.typesafe.akka" %% "akka-http" % akkaHttpVersion,
//      //      "com.typesafe.akka" %% "akka-http2-support" % akkaHttpVersion,
//
//      "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test,
//      "com.typesafe.akka" %% "akka-stream-testkit" % akkaVersion % Test,
//      "org.scalatest" %% "scalatest" % scalatestVersion % Test,
//    ),
//  )

lazy val front = (project in file("front"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  //  .aggregate(dtos.js, dtos.jvm)
//  .dependsOn(dtos)
  .settings(
    name := "my-health-record.ui",
    scalaVersion := scala3Version,
    Compile / PB.targets := Seq(
      scalapb.gen(grpc = false) -> (Compile / sourceManaged).value / "scalapb",
      scalapb.grpcweb.GrpcWebCodeGenerator -> (Compile / sourceManaged).value,
    ),
    Compile / npmDevDependencies  ++= Seq(
      "autoprefixer" -> "10.2.4",
      "tailwindcss" -> "2.0.2",
      "postcss" -> "8.2.4",
      "postcss-cli" -> "8.3.1",
    ),
    Compile / scalaJSLinkerConfig ~= { _.withSourceMap(false) },
    fullOptJS / scalaJSLinkerConfig ~= { _.withSourceMap(false) },
    scalaJSUseMainModuleInitializer := true,
    webpackBundlingMode := BundlingMode.LibraryAndApplication(),
    libraryDependencies ++= Seq(
      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion,
      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      "com.thesamet.scalapb.grpcweb" %%% "scalapb-grpcweb" % scalapb.grpcweb.BuildInfo.version,
    ),
    libraryDependencies ++= Seq(
      "com.raquo" %%% "laminar" % "0.13.0",
      "io.frontroute" %%% "frontroute" % "0.13.1",
      "com.outr" %%% "scribe" % "3.5.5",
//      "org.wvlet.airframe" %%% "airframe-log" % "21.4.1",
    ),
    libraryDependencies ++= Seq(
      "org.scalatest" %%% "scalatest" % scalatestVersion % Test,
    ),
  )


logLevel := Level.Debug

import sbt.Keys.streams
import scalajsbundler.BundlerFile.WebpackConfig

lazy val css = taskKey[Unit]("Compilal el CSS")
css := {
  //val result = npmInstallDependencies.value
  val logger = streams.value.log
  logger.info("1=================================>")
  logger.info(s"${front}")
  logger.info(s"${front.base}")
  logger.info(s"${WebpackConfig}")
  logger.info(s"${webpackConfigFile}")
  logger.info("1=================================<")
  //  logger.info("2=================================>")
  //  Npm.run("exec", "tailwindcss", "build", "-o tailwind.css")(front.base / "target" / "scala-2.13" / "scalajs-bundler" / "main", logger)
  Npm.run("exec", "--", "postcss",
    "--config", ((front / baseDirectory).value / "postcss.config.js").getAbsolutePath,
    "-o", ((front / baseDirectory).value / "target" / "compiled.css").getAbsolutePath,
    (front.base / "styles.css").getAbsolutePath)((front / crossTarget).value / "scalajs-bundler" / "main", logger)
  //  logger.info("2=================================<")
}
