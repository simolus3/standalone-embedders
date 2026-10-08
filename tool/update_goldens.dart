import 'dart:io';

/// Runs each Dart program in `tests/` with the Dart VM and records its stdout
/// in `tests/goldens/<program>.txt`.
void main() async {
  final tests = Directory('tests/vm_comparison');
  final output = Directory('tests/goldens');
  if (await output.exists()) {
    await output.delete(recursive: true);
  }
  await output.create(recursive: true);

  await for (final entity in tests.list()) {
    if (entity is! File || !entity.path.endsWith('.dart')) continue;

    final name = entity.uri.pathSegments.last;
    final result = await Process.run(Platform.resolvedExecutable, [
      'run',
      entity.path,
    ]);

    if (result.exitCode != 0) {
      stderr
        ..writeln('$name exited with code ${result.exitCode}:')
        ..write(result.stderr);
      exitCode = 1;
      continue;
    }

    await File('${output.path}/$name.txt').writeAsString(result.stdout);
    print('Updated golden for $name');
  }
}
