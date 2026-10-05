package eu.simonbinder.build;

import org.gradle.api.JavaVersion;
import org.gradle.api.Plugin;
import org.gradle.api.Project;
import org.gradle.api.plugins.JavaPluginExtension;
import org.jspecify.annotations.NonNull;

public class CommonJavaPlugin implements Plugin<Project> {
    @Override
    public void apply(@NonNull Project target) {
        target.getPluginManager().apply("java");
        target.getExtensions().configure(JavaPluginExtension.class, extension -> {
            var version = JavaVersion.VERSION_17;

            extension.setTargetCompatibility(version);
            extension.setSourceCompatibility(version);
        });
    }
}
