import 'dart:convert';

void main() {
  print(3.0);
  print(double.parse('3.0'));

  print(double.infinity);
  print(double.negativeInfinity);
  print(double.nan);
  print(0);
  print(-0);

  print(1.125.toStringAsFixed(2));

  print(json.decode('1.25'));

  // Exact ties (like 2.5.toStringAsExponential(0)) are not tested since we
  // round them to even instead of up.
  for (final value in [
    0.0,
    -0.0,
    1.0,
    -1.5,
    123.456,
    0.000123,
    1e21,
    1.7976931348623157e308,
    5e-324,
    1e-200,
    9.96,
    -99.96,
  ]) {
    print(value.toStringAsExponential());
    for (final digits in [0, 1, 3, 20]) {
      print(value.toStringAsExponential(digits));
    }
    for (final precision in [1, 2, 3, 7, 21]) {
      print(value.toStringAsPrecision(precision));
    }
  }
  print(0.0000001234.toStringAsPrecision(3));
  print(0.000001234.toStringAsPrecision(3));
  print(123456.0.toStringAsPrecision(6));
  print(123456.0.toStringAsPrecision(5));
}
