import 'dart:io';
import 'dart:math';

import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:cli_util/cli_util.dart';
import 'package:path/path.dart' as p;

void main() {
  final rustSrc = File('dart2wasmtime/src/lib.rs').readAsStringSync();
  final parsed = parseFile(
    path: p.join(
      sdkPath!,
      'lib',
      '_internal',
      'wasm',
      'standalone',
      'embedder.dart',
    ),
    featureSet: .latestLanguageVersion(),
  ).unit;

  bool supportedByRust(String import) {
    return rustSrc.contains('"$import"');
  }

  final names = <String>[];
  var longestNameLength = 0;

  for (final definition in parsed.declarations) {
    if (definition is! FunctionDeclaration) continue;

    for (final annotation in definition.metadata) {
      if (annotation.arguments?.arguments
          case [
            SimpleStringLiteral(value: 'wasm:import'),
            SimpleStringLiteral(:final value),
          ]
          when annotation.name.name == 'pragma') {
        final importName = value.substring(value.indexOf('.') + 1);
        names.add(importName);
        longestNameLength = max(longestNameLength, importName.length);
      }
    }
  }

  // Names are wrapped in backticks, which adds two characters.
  final nameWidth = max(longestNameLength + 2, 'Symbol'.length);

  final table = StringBuffer()
    ..writeln('| ${'Symbol'.padRight(nameWidth)} | Supported |')
    ..writeln('| ${'-' * nameWidth} | --------- |');
  for (final name in names) {
    final status = supportedByRust(name) ? '✅' : '❌';
    table.writeln('| ${'`$name`'.padRight(nameWidth)} | $status |');
  }

  final readme = File('README.md');
  final contents = readme.readAsStringSync();
  final start = RegExp(r'<!-- symbols-table:start.*-->\n').firstMatch(contents);
  final end = contents.indexOf('<!-- symbols-table:end -->');
  if (start == null || end == -1 || end < start.end) {
    stderr.writeln('Could not find symbols-table anchors in README.md');
    exit(1);
  }

  readme.writeAsStringSync(contents.replaceRange(start.end, end, '$table'));
}
