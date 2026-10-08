import 'dart:math';

void main() {
  print(3.0);
  print(double.parse('3.0'));

  print(double.infinity);
  print(double.negativeInfinity);
  print(double.nan);
  print(0);
  print(-0);

  // We just need this to not crash
  Random().nextDouble();
  Random.secure().nextDouble();
}
