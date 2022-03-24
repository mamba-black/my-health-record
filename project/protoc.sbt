addSbtPlugin("com.lightbend.akka.grpc" % "sbt-akka-grpc" % "2.1.4")
//addSbtPlugin("com.lightbend.sbt" % "sbt-javaagent" % "0.1.5")

addSbtPlugin("com.thesamet" % "sbt-protoc" % "1.0.5")
libraryDependencies += "com.thesamet.scalapb" %% "compilerplugin" % "0.11.10" // "0.11.7" //"0.11.3" //0.11.2
libraryDependencies += "com.thesamet.scalapb.grpcweb" %% "scalapb-grpcweb-code-gen" % "0.6.4" //0.6.4
