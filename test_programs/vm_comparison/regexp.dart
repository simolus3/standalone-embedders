void main() {
  final text = 'Hello world, hello 🦀 crab';

  final word = RegExp(r'\w+');
  for (final match in word.allMatches(text)) {
    print('${match.start}-${match.end}: ${match[0]}');
  }

  final groups = RegExp(r'(h)(e)(x)?llo');
  final match = groups.firstMatch(text)!;
  print([match.groupCount, match[1], match[2], match[3]]);

  final named = RegExp(r'(?<animal>🦀) (?<name>\w+)');
  final namedMatch = named.firstMatch(text)!;
  print(namedMatch.groupNames.toList()..sort());
  print([namedMatch.namedGroup('animal'), namedMatch.namedGroup('name')]);
  print([namedMatch.start, namedMatch.end]);

  print(RegExp('HELLO', caseSensitive: false).allMatches(text).length);
  print(RegExp('HELLO').hasMatch(text));
  print(RegExp(r'^b', multiLine: true).hasMatch('a\nb'));

  print(RegExp('world').matchAsPrefix(text, 6)?[0]);
  print(RegExp('world').matchAsPrefix(text, 5));

  print(text.replaceAll(RegExp(r'[aeiou]'), '_'));
  print(RegExp.escape('a.b*c'));

  try {
    RegExp('('); // ignore: valid_regexps
    print('no error');
  } on FormatException {
    print('FormatException');
  }
}
