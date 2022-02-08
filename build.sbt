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
  "-Xsource:3"
  // "-P:silencer:pathFilters=.*[/]src_managed[/].*"
  // "-Wconf:src=src_managed/.*:silent"
)

val scala2Version = "2.13.7"
val scala3Version = "3.1.0"

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

lazy val front = (project in file("front"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  //  .aggregate(dtos.js, dtos.jvm)
  .dependsOn(dtos)
  .settings(commonTasks)
  .settings(
    name := "my-health-record.ui",
    scalaVersion := scala2Version,
    Compile / npmDevDependencies ++= Seq(
      "autoprefixer" -> "10.3.1",
      "tailwindcss" -> "2.2.7",
      "postcss" -> "8.3.6",
      "postcss-cli" -> "8.3.1",
      // "webpack-dev-server" -> "3.11.2",
    ),
    // Compile / scalaJSLinkerConfig ~= {
    //  _.withSourceMap(false)
    // },
    // fullOptJS / scalaJSLinkerConfig ~= {
    //  _.withSourceMap(false)
    // },
    scalaJSUseMainModuleInitializer := true,
    webpackBundlingMode := BundlingMode.LibraryAndApplication(),
    webpackDevServerExtraArgs := Seq("--inline", "--host", "0.0.0.0", "--history-api-fallback"),
    libraryDependencies ++= Seq(
      "com.raquo" %%% "laminar" % "0.14.2",
      "io.frontroute" %%% "frontroute" % "0.15.2",
      "com.outr" %%% "scribe" % "3.6.4",
      "io.github.cquiroz" %%% "scala-java-time" % "2.3.0",
      "io.github.cquiroz" %%% "scala-java-time-tzdb" % "2.3.0",
      // "org.wvlet.airframe" %%% "airframe" % "21.6.0",
      // "org.wvlet.airframe" %%% "airframe-log" % "21.4.1",
    ),
    libraryDependencies ++= Seq("org.scalatest" %%% "scalatest" % scalatestVersion % Test),
  )

lazy val back = (project in file("back"))
  .enablePlugins(AkkaGrpcPlugin, JibPlugin)
  .settings(
    name := "my-health-record.back",
    scalaVersion := scala2Version,
    ThisBuild / dynverSeparator := "-",
    ThisBuild / scalacOptions ++= Seq("-Xsource:3"),
    jibBaseImage := "adoptopenjdk/openjdk11:alpine-jre",
    jibName := "pocs",
    jibOrganization := "miclaro",
    jibRegistry := "957838095201.dkr.ecr.us-east-1.amazonaws.com",
    // dockerBaseImage := "ghcr.io/graalvm/graalvm-ce:latest", // Tenemos problemas para detectar el maximo de memoria
    libraryDependencies ++= Seq(
      algoliaScala,
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
      // "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
      scalaTest,
      akkaActorTypedTest,
      akkaStreamTest,
      akkaPersistenceTest,
      "org.bouncycastle" % "bcprov-jdk16" % "1.46" % Test,
    ),
  )

lazy val testSuite = (project in file("test-suite"))
  .enablePlugins(GatlingPlugin)
  .settings(
    scalaVersion := scala2Version,
    ThisBuild / scalacOptions ++= Seq("-Xsource:3"),
    libraryDependencies ++= Seq(
      gatlingTest,
      gatlingChartsTest,
      gatlingGrpcTest,
      scalapbNetty,
      scalapbRuntime,
      scalapbRuntimeGrpc,
    ),
    Test / PB.targets := Seq(scalapb.gen() -> (Test / sourceManaged).value),
  )

logLevel := Level.Debug

import sbt.Keys.streams
import scalajsbundler.BundlerFile.WebpackConfig

lazy val css = inputKey[Unit]("Compilal el CSS")
lazy val commonTasks = Def.settings(css := {
  // val result = npmInstallDependencies.value
  val logger = streams.value.log
  logger.info("1=================================>")
  logger.info(s"$thisProject")
  logger.info(s"${(thisProject / baseDirectory).value}")
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
    ((thisProject / baseDirectory).value / "postcss.config.js").getAbsolutePath,
    "-o",
    // ((front / baseDirectory).value / "target" / "compiled.css").getAbsolutePath,
    ((thisProject / crossTarget).value / "scalajs-bundler" / "main" / "compiled.css").getAbsolutePath,
    ((thisProject / baseDirectory).value / "styles.css").getAbsolutePath,
  )((thisProject / crossTarget).value / "scalajs-bundler" / "main", logger)
  //  logger.info("2=================================<")

  IO.copyFile(
    java.nio.file.Path.of("front/index.html").toFile,
    ((thisProject / crossTarget).value / "scalajs-bundler" / "main" / "index.html").getAbsoluteFile,
  )
  IO.copyDirectory(
    java.nio.file.Path.of("front/public").toFile,
    ((thisProject / crossTarget).value / "scalajs-bundler" / "main" / "public").getAbsoluteFile,
  )
})

//(css in css) := ((css in css) dependsOn npmInstallDependencies).value

//val silencerVersion = "1.7.6"
//ThisBuild / libraryDependencies ++= Seq(
//  compilerPlugin("com.github.ghik" % "silencer-plugin" % silencerVersion cross CrossVersion.full),
//  "com.github.ghik" % "silencer-lib" % silencerVersion % Provided cross CrossVersion.full
//)
