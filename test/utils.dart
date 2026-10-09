import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:test/test.dart';

Future<ProcessResult> runProcess(
  String executable,
  List<String> arguments,
) async {
  final process = await Process.run(executable, arguments);

  if (process.exitCode != 0) {
    throw 'Could not run $executable ${arguments.join(', ')}: '
        '${process.stdout} / ${process.stderr}';
  }

  return process;
}

Future<String> cargoExampleBinary(String name) async {
  await runProcess('cargo', ['build', '--example', name]);

  final metadataOut = await runProcess('cargo', [
    'metadata',
    '--format-version',
    '1',
    '--no-deps',
  ]);
  final metadata =
      json.decode(metadataOut.stdout as String) as Map<String, Object?>;
  final targetDir = metadata['target_directory'] as String;

  return p.join(
    targetDir,
    'debug',
    'examples',
    Platform.isWindows ? '$name.exe' : name,
  );
}

final class CompiledModuleTest {
  late Directory scratchSpace;
  late String wasmPath;

  CompiledModuleTest(File input) {
    setUpAll(() async {
      scratchSpace = await Directory.systemTemp.createTemp(
        'dart-wasm-embedder',
      );
      wasmPath = p.join(scratchSpace.path, 'compiled.wasm');
      await runProcess(Platform.resolvedExecutable, [
        'compile',
        'wasm',
        '--standalone',
        '-E--no-strip-wasm',
        input.path,
        '-o',
        wasmPath,
      ]);
    });

    tearDownAll(() async {
      await scratchSpace.delete(recursive: true);
    });
  }
}
