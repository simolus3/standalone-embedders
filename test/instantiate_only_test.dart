import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:test/test.dart';

import 'utils.dart';

void main() {
  final sources = Directory('test_programs/instantiate_only').listSync();
  late String exampleRunner;

  setUpAll(() async {
    exampleRunner = await cargoExampleBinary('run');
  });

  for (final source in sources.whereType<File>()) {
    if (p.extension(source.path) == '.dart') {
      group(p.basename(source.path), () {
        final module = CompiledModuleTest(source);

        test('dart2wasmtime', () async {
          await runProcess(exampleRunner, [
            '--instantiate-only',
            module.wasmPath,
          ]);
        });
      });
    }
  }
}
