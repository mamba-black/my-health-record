addSbtPlugin("com.thesamet" % "sbt-protoc" % "1.0.0")
addSbtPlugin("com.lightbend.akka.grpc" % "sbt-akka-grpc" % "1.0.2")
//addSbtPlugin("com.lightbend.sbt" % "sbt-javaagent" % "0.1.5")

libraryDependencies += "com.thesamet.scalapb" %% "compilerplugin" % "0.10.11"
libraryDependencies += "com.thesamet.scalapb.grpcweb" %% "scalapb-grpcweb-code-gen" % "0.4.2"
