import org.scalajs.sbtplugin.ScalaJSPlugin.autoImport.scalaJSUseMainModuleInitializer
import scalajsbundler.Npm

ThisBuild / scalaVersion := "2.13.4"
//ThisBuild / scalaVersion := "3.0.0-M3"
ThisBuild / organization := "miuler"

lazy val dtos = (project in file("dtos"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  .settings(
    name := "my-health-record.dtos",
    scalaJSUseMainModuleInitializer := true,
  )

lazy val front = (project in file("front"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
//  .aggregate(dtos.js, dtos.jvm)
  .dependsOn(dtos)
  .settings(
    name := "my-health-record.ui",
    Compile / PB.targets := Seq(
      scalapb.gen(grpc=false) -> (Compile / sourceManaged).value / "scalapb",
      scalapb.grpcweb.GrpcWebCodeGenerator -> (Compile / sourceManaged).value,
    ),
    scalaJSUseMainModuleInitializer := true,
    npmDevDependencies in Compile += "autoprefixer" -> "10.2.4",
    npmDevDependencies in Compile += "tailwindcss" -> "2.0.2",
    npmDevDependencies in Compile += "postcss" -> "8.2.4",
    npmDevDependencies in Compile += "postcss-cli" -> "8.3.1",
    libraryDependencies += "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion,
    libraryDependencies += "com.thesamet.scalapb" %%% "scalapb-runtime" % scalapb.compiler.Version.scalapbVersion % "protobuf",
    libraryDependencies += "com.thesamet.scalapb.grpcweb" %%% "scalapb-grpcweb" % scalapb.grpcweb.BuildInfo.version,
    libraryDependencies ++= Seq(
      "com.raquo" %%% "laminar" % "0.11.0",
      "com.outr" %%% "scribe" % "3.1.9",
//      "org.wvlet.airframe" %%% "airframe-log" % "21.1.0",
      "org.scalatest" %%% "scalatest" % "3.2.3" % Test
    ),
  )

logLevel := Level.Debug
import sbt.Keys.streams
import scalajsbundler.BundlerFile.WebpackConfig
lazy val css = taskKey[Unit]("Compilal el CSS")
css := {
//  val logger: Logger = ConsoleLogger()
  val logger = streams.value.log
  npmInstallDependencies
  logger.info("1=================================>")
  logger.info(s"${front}")
  logger.info(s"${front.base}")
  logger.info(s"${WebpackConfig}")
  logger.info(s"${webpackConfigFile}")
  logger.info("1=================================<")
//  logger.info("2=================================>")
//  Npm.run("exec", "tailwindcss", "build", "-o tailwind.css")(front.base / "target" / "scala-2.13" / "scalajs-bundler" / "main", logger)
  Npm.run("exec", "--", "postcss",
    "--config", (baseDirectory.in(front).value / "postcss.config.js").getAbsolutePath,
    "-o", (baseDirectory.in(front).value / "target" / "scala-2.13" / "my-health-record-ui-fastopt" / "compiled.css").getAbsolutePath,
    (baseDirectory.in(front).value / "styles.css").getAbsolutePath)(front.base / "target" / "scala-2.13" / "scalajs-bundler" / "main", logger)
//  logger.info("2=================================<")
}
