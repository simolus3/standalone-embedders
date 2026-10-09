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
}
