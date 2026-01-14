name := "WienerProcess"

version := "1.0"

scalaVersion := "2.12.12"

libraryDependencies += "org.scalatest" %% "scalatest" % "3.2.3" % Test

scalaSource in Compile := baseDirectory.value / "src" / "main" / "scala"

mainClass in Compile := Some("WienerProcess")

target in assembly := file("lib")

assemblyMergeStrategy in assembly := {
  case PathList("META-INF", xs @ _*) => MergeStrategy.discard
  case x                             => MergeStrategy.first
}
