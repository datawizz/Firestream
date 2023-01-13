name := "kafka-streams-stateful"

version := "1.0"

scalaVersion := "2.12.12"

resolvers += "Maven Central" at "https://repo1.maven.org/maven2"
resolvers += "sbt Plugin Releases" at "https://repo.scala-sbt.org/scalasbt/sbt-plugin-releases"
resolvers += "Typesafe Releases" at "https://repo.typesafe.com/typesafe/ivy-releases"

libraryDependencies ++= Seq(
  "org.apache.kafka" % "kafka-streams-scala_2.12" % "3.3.1",
  "org.scalatest" % "scalatest_2.12" % "3.2.3" % Test,
  "org.json" % "json" % "20220924",
  "org.json4s" %% "json4s-native" % "4.1.0-M2",
  "org.json4s" %% "json4s-jackson" % "4.1.0-M2",
  "io.circe" %% "circe-core" % "0.15.0-M1",
  "io.circe" %% "circe-parser" % "0.15.0-M1",
  "io.circe" %% "circe-generic" % "0.15.0-M1"
)

scalaSource in Compile := baseDirectory.value / "src" / "main" / "scala"

mainClass in Compile := Some("Main")

assemblyMergeStrategy in assembly := {
  case PathList("META-INF", xs @ _*) => MergeStrategy.discard
  case x                             => MergeStrategy.first
}
