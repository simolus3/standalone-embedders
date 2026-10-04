rootProject.name = "endive-dart"
enableFeaturePreview("TYPESAFE_PROJECT_ACCESSORS")

includeBuild("build-logic")

dependencyResolutionManagement {
    @Suppress("UnstableApiUsage")
    repositories {
        mavenCentral()
    }
}

include("runtime")
include("examples:hello_world")
include("examples:swing")
