plugins {
    `java-gradle-plugin`
    alias(libs.plugins.spotless)
}

java {
    sourceCompatibility = JavaVersion.VERSION_25
    targetCompatibility = JavaVersion.VERSION_25
}

dependencies {
    compileOnly("run.endive:build-time-compiler:1.1.0")
    implementation(libs.spotless.plugin)
}

spotless {
    java {
        // google-java-format indents with two spaces.
        googleJavaFormat(libs.versions.googleJavaFormat.get())
    }
}

gradlePlugin {
    plugins {
        create("example") {
            id = "eu.simonbinder.endive.example"
            implementationClass = "eu.simonbinder.build.ExamplePlugin"
        }

        create("library") {
            id = "eu.simonbinder.endive.library"
            implementationClass = "eu.simonbinder.build.JavaLibraryPlugin"
        }
    }
}
