plugins {
    id("eu.simonbinder.endive.library")
}

group = "eu.simonbinder"

dependencies {
    api(libs.endive.run) {
        isChanging = true
    }

    testImplementation(platform("org.junit:junit-bom:6.0.0"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}
