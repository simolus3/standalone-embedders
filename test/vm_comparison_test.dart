import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:test/test.dart';

import 'utils.dart';

/// Runs a test program on the Dart VM, then compiles it to WebAssembly at runs
/// it through an embedder to compare outputs.
void main() {
  final sources = Directory('test_programs/vm_comparison').listSync();
  late String exampleRunner;

  setUpAll(() async {
    exampleRunner = await cargoExampleBinary('run');
  });

  for (final source in sources.whereType<File>()) {
    if (p.extension(source.path) == '.dart') {
      group(
        p.basename(source.path),
        () => _defineComparisonTest(() => exampleRunner, source),
      );
    }
  }
}

void _defineComparisonTest(String Function() exampleRunner, File input) {
  final module = CompiledModuleTest(input);
  late String referenceOutput;

  setUpAll(() async {
    final output = await runProcess(Platform.resolvedExecutable, [input.path]);
    if ((output.stderr as String).isNotEmpty) {
      throw 'Unexpected stderr: ${output.stderr}';
    }

    referenceOutput = output.stdout as String;
  });

  test('dart2wasmtime', () async {
    final out = await runProcess(exampleRunner(), [module.wasmPath]);
    expect(out.stderr, isEmpty);
    expect(out.stdout, referenceOutput);
  });
}
