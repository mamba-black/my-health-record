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
  "-Wconf:src=src_managed/.*:silent",
)

val scala2Version = "2.13.6"
val scala3Version = "3.0.0"

lazy val dtos = project
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  .in(file("dtos"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  .settings(
    name := "my-health-record.dtos",
    scalaVersion := scala2Version,
    scalaJSUseMainModuleInitializer := true,
    Compile / PB.targets := Seq(
      scalapb.gen(grpc = false) -> (Compile / sourceManaged).value / "scalapb",
      scalapb.grpcweb.GrpcWebCodeGenerator -> (Compile / sourceManaged).value,
    ),
    libraryDependencies ++= Seq(
      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion,
      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      "com.thesamet.scalapb.grpcweb" %%% "scalapb-grpcweb" % scalapb.grpcweb.BuildInfo.version,
    ),
  )

lazy val akkaVersion = "2.6.15"
lazy val akkaHttpVersion = "10.2.4"
//lazy val algolia = "3.14.1"
lazy val scalatestVersion = "3.2.9"

lazy val back = (project in file("back"))
  .enablePlugins(AkkaGrpcPlugin, JavaAppPackaging, DockerPlugin)
  .settings(
    name := "my-health-record.back",
    scalaVersion := scala2Version,
    ThisBuild / dynverSeparator := "-",
    //dockerBaseImage := "ghcr.io/graalvm/graalvm-ce:latest", // Tenemos problemas para detectar el maximo de memoria
    // //Docker / packageName := "gcr.io/miuler-medical-001",
    //dockerRepository := Some("gcr.io/miuler-medical-001"),
    dockerBaseImage := "adoptopenjdk/openjdk11:debianslim-jre",
    dockerRepository := Some("957838095201.dkr.ecr.us-east-1.amazonaws.com/miclaro"),
    dockerExposedPorts := Seq(8080),
    dockerUpdateLatest := true,
    Docker / packageName := "pocs",

    // COLOCAR LO SIGUIENTE PARA QUE NO DE ERROR ===========>
    Docker / daemonUserUid := None,
    Docker / daemonUser := "daemon",
    // COLOCAR LO SIGUIENTE PARA QUE NO DE ERROR ===========>

    libraryDependencies ++= Seq(
      "com.typesafe.akka" %% "akka-http" % akkaHttpVersion,
      "com.typesafe.akka" %% "akka-actor-typed" % akkaVersion,
      "com.typesafe.akka" %% "akka-stream" % akkaVersion,
      "com.typesafe.akka" %% "akka-discovery" % akkaVersion,
      "com.typesafe.akka" %% "akka-persistence-typed" % akkaVersion,

      //      "com.typesafe.akka" %% "akka-pki" % akkaVersion,
      "com.lightbend.akka.grpc" %% "akka-grpc-runtime" % "2.0.0",
      "ch.megard" %% "akka-http-cors" % "1.1.1", // Para poder usar akka grpc con grpc-web

      "ch.qos.logback" % "logback-classic" % "1.2.3",
      "com.outr" %% "scribe" % "3.5.5",

      //"com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      //"com.algolia" % "algoliasearch-core" % algolia,
      //"com.algolia" % "algoliasearch-java-net" % algolia,

      "org.scalatest" %% "scalatest" % scalatestVersion % Test,
      "com.typesafe.akka" %% "akka-actor-testkit-typed" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-stream-testkit" % akkaVersion % Test,
      "com.typesafe.akka" %% "akka-persistence-testkit" % akkaVersion % Test,
    ),
  )

lazy val front = (project in file("front"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  //  .aggregate(dtos.js, dtos.jvm)
  .dependsOn(dtos)
  .settings(
    name := "my-health-record.ui",
    scalaVersion := scala2Version,
    Compile / npmDevDependencies ++= Seq(
      "autoprefixer" -> "10.2.4",
      "tailwindcss" -> "2.0.2",
      "postcss" -> "8.2.4",
      "postcss-cli" -> "8.3.1",
      //"webpack-dev-server" -> "3.11.2",
    ),
    //Compile / scalaJSLinkerConfig ~= {
    //  _.withSourceMap(false)
    //},
    //fullOptJS / scalaJSLinkerConfig ~= {
    //  _.withSourceMap(false)
    //},
    scalaJSUseMainModuleInitializer := true,
    webpackBundlingMode := BundlingMode.LibraryAndApplication(),
    webpackDevServerExtraArgs := Seq("--inline", "--hot", "--history-api-fallback"),
    libraryDependencies ++= Seq(
      "com.raquo" %%% "laminar" % "0.13.0",
      "io.frontroute" %%% "frontroute" % "0.13.2",
      "com.outr" %%% "scribe" % "3.5.5",
      "io.github.cquiroz" %%% "scala-java-time" % "2.3.0",
      "io.github.cquiroz" %%% "scala-java-time-tzdb" % "2.3.0",
      //"org.wvlet.airframe" %%% "airframe" % "21.6.0",
      //"org.wvlet.airframe" %%% "airframe-log" % "21.4.1",
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
