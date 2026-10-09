import 'dart:convert';

/// Prevents the compiler from constant-folding string operations.
@pragma('wasm:never-inline')
@pragma('vm:never-inline')
String opaque(String s) => s;

void main() {
  final hello = opaque('Hello world, hello wasm');
  final crab = opaque('crab 🦀 and ☕️ and 🦀 again');

  print(hello.indexOf('hello'));
  print(hello.indexOf('o', 5));
  print(hello.indexOf('missing'));
  print(hello.lastIndexOf('o'));
  print(hello.lastIndexOf('o', 6));
  print(hello.lastIndexOf('Hello', 0));
  print(hello.lastIndexOf('this pattern is much longer than the string'));
  print(crab.indexOf('and'));
  print(crab.lastIndexOf('🦀'));
  print(crab.indexOf('again', 10));
  print(crab.length);
  print(crab.codeUnitAt(5));
  print(crab.codeUnitAt(6));

  print(hello.replaceAll('o', '0'));
  print(hello.replaceAll(opaque('hello'), opaque('bye')));
  print(crab.replaceAll('🦀', 'crab'));

  print(hello.substring(6));
  print(hello.substring(0, 5));
  print(crab.substring(5, 7));
  print(crab.substring(8));

  print(hello.toLowerCase());
  print(hello.toUpperCase());
  print(opaque('already lower').toLowerCase());
  print(opaque('ÄÖÜ äöü').toLowerCase());
  print(opaque('ÄÖÜ äöü').toUpperCase());

  print(hello + opaque('!'));
  print(crab + opaque(' ') + hello);

  print(opaque('ab') * 3);
  print(opaque('🦀') * 2);

  print(hello.replaceRange(0, 5, 'Goodbye'));
  print(crab.replaceRange(5, 7, 'CRAB'));
  print(hello.replaceRange(5, 5, ','));

  print(jsonDecode(opaque('"héllo 🦀 \\u00e9"')));
  print(jsonDecode(opaque('{"key": ["a", "b"]}')));
}
