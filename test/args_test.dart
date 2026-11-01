import 'dart:io';

import 'package:test/test.dart';
import 'package:test_descriptor/test_descriptor.dart' as d;

import 'utils.dart';

void main() {
  late String exampleRunner;

  setUpAll(() async {
    exampleRunner = await cargoExampleBinary('run');
  });

  test('can pass arguments to program', () async {
    // Regression test for https://github.com/simolus3/standalone-embedders/pull/3
    final wasm = d.path('test.wasm');
    await runProcess(Platform.resolvedExecutable, [
      'compile',
      'wasm',
      '--standalone',
      '-E--no-strip-wasm',
      'test_programs/use_args.dart',
      // Skip wasm_opt as it breaks argument passing at the moment
      '--phases=cfe,tfa,codegen',
      '-o',
      wasm,
    ]);

    final out = await runProcess(exampleRunner, [wasm, '--', 'hello', 'world']);
    expect(out.stdout, 'hello\nworld\n');
  });
}
