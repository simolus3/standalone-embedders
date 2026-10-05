package eu.simonbinder.build;

import org.gradle.api.Plugin;
import org.gradle.api.Project;
import org.gradle.api.attributes.Usage;
import org.gradle.api.plugins.JavaPluginExtension;
import org.jspecify.annotations.NonNull;

public class ExamplePlugin implements Plugin<Project> {
    @Override
    public void apply(@NonNull Project target) {
        var plugins = target.getPlugins();
        plugins.apply(JavaLibraryPlugin.class);
        plugins.apply("application");

        target.getConfigurations().create(COMPILER_RUNTIME_CLASSPATH, spec -> {
            spec.setCanBeResolved(true);
            spec.setCanBeConsumed(false);
            spec.setDescription("Classpath for the endive build time compiler");
            spec.getAttributes().attribute(Usage.USAGE_ATTRIBUTE, target.getObjects().named(Usage.class, Usage.JAVA_RUNTIME));
        });

        var dartSdk = target.getProviders().gradleProperty(DART_SDK_PROPERTY);
        var compileDart = target.getTasks().register("compileDartToWasm", CompileDartToWasm.class, task -> {
            task.getDartSdk().set(dartSdk);
            task.getSources().from(target.fileTree(target.getProjectDir(), tree -> {
                tree.include("**/*.dart");
                tree.exclude("build/**", ".dart_tool/**");
            }));
            task.getOutputFile().convention(target.getLayout().getBuildDirectory().file("dart-wasm/main.wasm"));
        });

        var taskRef = target.getTasks().register("compileWebAssembly", PreCompileWebAssembly.class, task -> {
            task.getWebAssemblyFile().convention(compileDart.flatMap(CompileDartToWasm::getOutputFile));
        });

        target.getExtensions().configure(JavaPluginExtension.class, java -> {
            java.getSourceSets().named("main", sourceSet -> {
                sourceSet.getJava().srcDir(taskRef.flatMap(PreCompileWebAssembly::getOutputSources));
                sourceSet.getResources().srcDir(taskRef.flatMap(PreCompileWebAssembly::getOutputResources));
            });
        });

        target.getDependencies().add("implementation", target.files(
                taskRef.flatMap(PreCompileWebAssembly::getOutputClasses)
        ));
    }

    public static final String DART_SDK_PROPERTY = "dartSdk";
    public static final String COMPILER_RUNTIME_CLASSPATH = "endiveCompilerRuntimeClasspath";
}
