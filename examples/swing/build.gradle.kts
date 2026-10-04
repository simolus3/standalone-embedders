import eu.simonbinder.build.CompileDartToWasm

plugins {
    id("eu.simonbinder.endive.example")
}

application {
    mainClass.set("eu.simonbinder.endive.example.SwingCounter")
}

dependencies {
    implementation(projects.runtime)
    endiveCompilerRuntimeClasspath(libs.endive.compiler)
}

tasks.withType<CompileDartToWasm>().configureEach {
    entrypoint.set(file("counter.dart"))
}
