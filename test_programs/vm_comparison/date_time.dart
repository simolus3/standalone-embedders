void main() {
  for (final millis in [
    0,
    // Northern hemisphere winter and summer, to cover daylight saving time.
    1767225600000, // 2026-01-01
    1782864000000, // 2026-07-01
    -2208988800000, // 1900-01-01
  ]) {
    final date = DateTime.fromMillisecondsSinceEpoch(millis);
    print('$date ${date.timeZoneName} ${date.timeZoneOffset}');
  }

  final local = DateTime(2026, 3, 29, 12);
  print('$local ${local.millisecondsSinceEpoch} ${local.timeZoneName}');
  print(DateTime.utc(2026, 3, 29, 12).toLocal());
}
