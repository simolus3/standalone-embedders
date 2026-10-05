package eu.simonbinder.build;

import org.gradle.api.DefaultTask;
import org.gradle.api.file.ConfigurableFileCollection;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.file.RegularFileProperty;
import org.gradle.api.provider.Property;
import org.gradle.api.tasks.*;
import org.gradle.workers.WorkAction;
import org.gradle.workers.WorkParameters;
import org.gradle.workers.WorkerExecutor;
import run.endive.build.time.compiler.Config;
import run.endive.build.time.compiler.Generator;

import javax.inject.Inject;
import java.io.IOException;

import static eu.simonbinder.build.ExamplePlugin.COMPILER_RUNTIME_CLASSPATH;

public abstract class PreCompileWebAssembly extends DefaultTask {
    @Inject
    public PreCompileWebAssembly() {
        getModuleName().convention("eu.simonbinder.demo.CompiledDartApp");

        var project = getProject();
        var defaultDir = getModuleName().flatMap(name -> project.getLayout().getBuildDirectory().dir("compiled-wasm/" + name));

        getCompilerClasspath().convention(project.getConfigurations().getByName(COMPILER_RUNTIME_CLASSPATH));
        getOutputClasses().convention(defaultDir.map(dir -> dir.dir("classes")));
        getOutputSources().convention(defaultDir.map(dir -> dir.dir("sources")));
        getOutputResources().convention(defaultDir.map(dir -> dir.dir("resources")));
    }

    @InputFile
    public abstract RegularFileProperty getWebAssemblyFile();

    @OutputDirectory
    public abstract DirectoryProperty getOutputClasses();

    @OutputDirectory
    public abstract DirectoryProperty getOutputSources();

    @OutputDirectory
    public abstract DirectoryProperty getOutputResources();

    @InputFiles
    @Classpath
    public abstract ConfigurableFileCollection getCompilerClasspath();

    @Input
    public abstract Property<String> getModuleName();

    @Inject
    protected abstract WorkerExecutor getExecutor();

    @TaskAction
    public void run() {
        var queue = getExecutor().processIsolation(spec -> {
            spec.getClasspath().from(getCompilerClasspath());
        });

        queue.submit(GenerateBytecode.class, params -> {
            params.getWasm().set(getWebAssemblyFile());
            params.getName().set(getModuleName());
            params.getOutputClasses().set(getOutputClasses());
            params.getOutputSources().set(getOutputSources());
            params.getOutputResources().set(getOutputResources());
        });
    }

    abstract static class GenerateBytecode implements WorkAction<GenerateBytecodeParameters> {
        @Inject
        public GenerateBytecode() {}

        @Override
        public void execute() {
            var params = getParameters();
            var config = Config.builder()
                .withWasmFile(params.getWasm().getAsFile().get().toPath())
                .withName(params.getName().get())
                .withTargetClassFolder(params.getOutputClasses().getAsFile().get().toPath())
                .withTargetSourceFolder(params.getOutputSources().getAsFile().get().toPath())
                .withTargetWasmFolder(params.getOutputResources().getAsFile().get().toPath())
                .withMethodPrefixer("run.endive.compiler.NameSectionMethodPrefixer")
                .build();

            var generator = new Generator(config);
            try {
                generator.generateSources();
                var interpreted = generator.generateResources();
                generator.generateMetaWasm(interpreted);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }
    }

    interface GenerateBytecodeParameters extends WorkParameters {
        RegularFileProperty getWasm();
        Property<String> getName();

        DirectoryProperty getOutputClasses();
        DirectoryProperty getOutputSources();
        DirectoryProperty getOutputResources();
    }
}
