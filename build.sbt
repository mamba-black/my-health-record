import org.scalajs.sbtplugin.ScalaJSPlugin.autoImport.scalaJSUseMainModuleInitializer

ThisBuild / scalaVersion := "2.13.4"
//ThisBuild / scalaVersion := "3.0.0-M3"
ThisBuild / organization := "miuler"

lazy val dtos = (project in file("dtos"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  .settings(
    name := "my-health-record.dtos",
    scalaJSUseMainModuleInitializer := true,
  )

lazy val record = (project in file("front"))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
//  .aggregate(dtos.js, dtos.jvm)
  .dependsOn(dtos)
  .settings(
    name := "my-health-record.ui",
    scalaJSUseMainModuleInitializer := true,
    libraryDependencies ++= Seq(
      "com.raquo" %%% "laminar" % "0.11.0",
      "com.outr" %%% "scribe" % "3.1.9",
//      "org.wvlet.airframe" %%% "airframe-log" % "21.1.0",
      "org.scalatest" %%% "scalatest" % "3.2.3" % Test
    ),
  )
