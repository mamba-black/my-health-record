addSbtPlugin("com.thesamet" % "sbt-protoc" % "1.0.4")
//addSbtPlugin("com.lightbend.akka.grpc" % "sbt-akka-grpc" % "1.1.1")
//addSbtPlugin("com.lightbend.sbt" % "sbt-javaagent" % "0.1.5")

libraryDependencies += "com.thesamet.scalapb" %% "compilerplugin" % "0.11.3" //0.11.2
libraryDependencies += "com.thesamet.scalapb.grpcweb" %% "scalapb-grpcweb-code-gen" % "0.6.4" //0.6.4
