import Dependencies._
import org.scalajs.sbtplugin.ScalaJSPlugin.autoImport.scalaJSUseMainModuleInitializer
import scalajsbundler.Npm

resolvers += Resolver.sonatypeRepo("public")
ThisBuild / organization := "miuler"
//ThisBuild / libraryDependencies ++= Seq(
//  compilerPlugin("com.github.ghik" % "silencer-plugin" % "1.7.4" cross CrossVersion.full),
//  "com.github.ghik" % "silencer-lib" % "1.7.4" % Provided cross CrossVersion.full
//)
ThisBuild / scalacOptions ++= Seq(
  //"-P:silencer:pathFilters=.*[/]src_managed[/].*"
  "-Wconf:src=src_managed/.*:silent"
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
      scalapb.grpcweb.GrpcWebCodeGenerator -> (Compile / sourceManaged).value
    ),
    libraryDependencies ++= Seq(
      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion,
      "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      "com.thesamet.scalapb.grpcweb" %%% "scalapb-grpcweb" % scalapb.grpcweb.BuildInfo.version
    )
  )

lazy val front = (project in file("front"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  //  .aggregate(dtos.js, dtos.jvm)
  .dependsOn(dtos)
  .settings(
    name := "my-health-record.ui",
    scalaVersion := scala2Version,
    Compile / npmDevDependencies ++= Seq(
      "autoprefixer" -> "10.3.1",
      "tailwindcss" -> "2.2.7",
      "postcss" -> "8.3.6",
      "postcss-cli" -> "8.3.1"
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
      "com.raquo" %%% "laminar" % "0.13.1",
      "io.frontroute" %%% "frontroute" % "0.14.0",
      "com.outr" %%% "scribe" % "3.5.5",
      "io.github.cquiroz" %%% "scala-java-time" % "2.3.0",
      "io.github.cquiroz" %%% "scala-java-time-tzdb" % "2.3.0"
      //"org.wvlet.airframe" %%% "airframe" % "21.6.0",
      //"org.wvlet.airframe" %%% "airframe-log" % "21.4.1",
    ),
    libraryDependencies ++= Seq("org.scalatest" %%% "scalatest" % scalatestVersion % Test)
  )

lazy val back = (project in file("back"))
  .enablePlugins(AkkaGrpcPlugin, JibPlugin)
  .settings(
    name := "my-health-record.back",
    scalaVersion := scala2Version,
    ThisBuild / dynverSeparator := "-",
    jibBaseImage := "adoptopenjdk/openjdk11:alpine-jre",
    jibName := "pocs",
    jibOrganization := "miclaro",
    jibRegistry := "957838095201.dkr.ecr.us-east-1.amazonaws.com",
    //dockerBaseImage := "ghcr.io/graalvm/graalvm-ce:latest", // Tenemos problemas para detectar el maximo de memoria
    libraryDependencies ++= Seq(
      akkaHttp,
      akkaActorTyped,
      akkaStream,
      akkaDiscovery,
      akkaPersistenceTyped,
      akkaStreamKafka,
      akkaSlf4j,
      logbackClassic,
      scribe,
      akkaGrpcRuntime,
      akkaHttpCors,
      //"com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      scalaTest,
      akkaActorTypedTest,
      akkaStreamTest,
      akkaPersistenceTest
    )
  )

lazy val testSuite = (project in file("test-suite"))
  .enablePlugins(GatlingPlugin)
  .settings(
    scalaVersion := scala2Version,
    libraryDependencies ++= Seq(
      gatlingTest,
      gatlingChartsTest,
      gatlingGrpcTest,
      scalapbNetty,
      scalapbRuntime,
      scalapbRuntimeGrpc
    ),
    Test / PB.targets := Seq(scalapb.gen() -> (Test / sourceManaged).value)
  )

logLevel := Level.Debug

import sbt.Keys.streams
import scalajsbundler.BundlerFile.WebpackConfig

lazy val css = taskKey[Unit]("Compilal el CSS")
css := {
  //val result = npmInstallDependencies.value
  val logger = streams.value.log
  logger.info("1=================================>")
  logger.info(s"$front")
  logger.info(s"${front.base}")
  logger.info(s"$WebpackConfig")
  logger.info(s"$webpackConfigFile")
  logger.info("1=================================<")
  //  logger.info("2=================================>")
  //  Npm.run("exec", "tailwindcss", "build", "-o tailwind.css")(front.base / "target" / "scala-2.13" / "scalajs-bundler" / "main", logger)
  Npm.run(
    "exec",
    "--",
    "postcss",
    "--config",
    ((front / baseDirectory).value / "postcss.config.js").getAbsolutePath,
    "-o",
    //((front / baseDirectory).value / "target" / "compiled.css").getAbsolutePath,
    ((front / crossTarget).value / "scalajs-bundler" / "main" / "compiled.css").getAbsolutePath,
    (front.base / "styles.css").getAbsolutePath
  )((front / crossTarget).value / "scalajs-bundler" / "main", logger)
  //  logger.info("2=================================<")
}

val silencerVersion = "1.7.5"
ThisBuild / libraryDependencies ++= Seq(
  compilerPlugin("com.github.ghik" % "silencer-plugin" % silencerVersion cross CrossVersion.full),
  "com.github.ghik" % "silencer-lib" % silencerVersion % Provided cross CrossVersion.full
)
