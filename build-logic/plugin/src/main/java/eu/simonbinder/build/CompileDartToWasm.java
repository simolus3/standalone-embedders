package eu.simonbinder.build;

import org.gradle.api.DefaultTask;
import org.gradle.api.file.ConfigurableFileCollection;
import org.gradle.api.file.RegularFileProperty;
import org.gradle.api.provider.Property;
import org.gradle.api.tasks.*;
import org.gradle.process.ExecOperations;

import javax.inject.Inject;
import java.io.File;
import java.util.List;

/**
 * Compiles a Dart entrypoint to a WebAssembly module.
 */
public abstract class CompileDartToWasm extends DefaultTask {
    @Inject
    public CompileDartToWasm() {
        getDartSdk().convention("");
    }

    @InputFile
    @PathSensitive(PathSensitivity.RELATIVE)
    public abstract RegularFileProperty getEntrypoint();

    @InputFiles
    @PathSensitive(PathSensitivity.RELATIVE)
    public abstract ConfigurableFileCollection getSources();

    @Input
    public abstract Property<String> getDartSdk();

    @OutputFile
    public abstract RegularFileProperty getOutputFile();

    @Inject
    protected abstract ExecOperations getExecOperations();

    @TaskAction
    public void run() {
        var output = getOutputFile().get().getAsFile();
        getExecOperations().exec(spec -> {
            spec.setExecutable(dartExecutable());
            spec.setArgs(List.of(
                    "compile",
                    "wasm",
                    "--standalone",
                    "-E--no-strip-wasm",
                    "--no-source-maps",
                    "-E--enable-experimental-wasm-interop",
                    getEntrypoint().get().getAsFile().getAbsolutePath(),
                    "-o",
                    output.getAbsolutePath()
            ));
        });
    }

    private String dartExecutable() {
        var sdk = getDartSdk().get().trim();
        if (sdk.isEmpty()) {
            return "dart";
        }

        if (sdk.startsWith("~/")) {
            sdk = System.getProperty("user.home") + sdk.substring(1);
        }
        return new File(sdk, "bin/dart").getAbsolutePath();
    }
}
