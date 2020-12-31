ThisBuild / scalaVersion := "2.13.4"
ThisBuild / organization := "miuler"

lazy val record = (project in file("."))
  .enablePlugins(ScalaJSPlugin, ScalaJSBundlerPlugin)
  .settings(
    name := "my-health-record.ui",
    scalaJSUseMainModuleInitializer := true,
    libraryDependencies ++= Seq(
      "com.raquo" %%% "laminar" % "0.11.0",
      "com.outr" %%% "scribe" % "3.1.8",
      "org.scalatest" %%% "scalatest" % "3.2.3" % Test
    )
  )
