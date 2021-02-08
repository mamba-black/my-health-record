addSbtPlugin("com.thesamet" % "sbt-protoc" % "1.0.0")

libraryDependencies += "com.thesamet.scalapb" %% "compilerplugin" % "0.10.10"

// GPRC services ------------------------------------------------->
val grpcWebVersion = "0.4.2"
libraryDependencies += "com.thesamet.scalapb.grpcweb" %% "scalapb-grpcweb-code-gen" % grpcWebVersion
// GPRC services -------------------------------------------------<
