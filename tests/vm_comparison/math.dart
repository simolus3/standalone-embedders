import 'dart:math';

// Results of transcendental functions may differ in the last bits between
// implementations, so most checks compare against known values with a
// tolerance and print whether they're close enough.
const epsilon = 1e-12;

void check(String name, double actual, double expected) {
  final ok = actual == expected || (actual - expected).abs() < epsilon;
  print('$name: $ok');
}

void checkNaN(String name, double actual) {
  print('$name is NaN: ${actual.isNaN}');
}

@pragma('wasm:never-inline')
double v(num x) => x.toDouble();

void main() {
  // pow
  check('pow(2, 10)', pow(v(2), v(10)).toDouble(), 1024);
  check('pow(2, 0.5)', pow(v(2), v(0.5)).toDouble(), sqrt2);
  check('pow(9, 0.5)', pow(v(9), v(0.5)).toDouble(), 3);
  check('pow(2, -1)', pow(v(2), v(-1)).toDouble(), 0.5);
  check('pow(0, 0)', pow(v(0), v(0)).toDouble(), 1);
  check('pow(e, 1)', pow(v(e), v(1)).toDouble(), e);
  checkNaN('pow(-1, 0.5)', pow(v(-1), v(0.5)).toDouble());

  // atan2
  check('atan2(0, 1)', atan2(v(0), v(1)), 0);
  check('atan2(1, 1)', atan2(v(1), v(1)), pi / 4);
  check('atan2(1, 0)', atan2(v(1), v(0)), pi / 2);
  check('atan2(0, -1)', atan2(v(0), v(-1)), pi);
  check('atan2(-1, -1)', atan2(v(-1), v(-1)), -3 * pi / 4);

  // sin
  check('sin(0)', sin(v(0)), 0);
  check('sin(pi / 6)', sin(v(pi / 6)), 0.5);
  check('sin(pi / 2)', sin(v(pi / 2)), 1);
  check('sin(pi)', sin(v(pi)), 0);
  check('sin(-pi / 2)', sin(v(-pi / 2)), -1);
  checkNaN('sin(infinity)', sin(v(double.infinity)));

  // cos
  check('cos(0)', cos(v(0)), 1);
  check('cos(pi / 3)', cos(v(pi / 3)), 0.5);
  check('cos(pi / 2)', cos(v(pi / 2)), 0);
  check('cos(pi)', cos(v(pi)), -1);
  checkNaN('cos(infinity)', cos(v(double.infinity)));

  // tan
  check('tan(0)', tan(v(0)), 0);
  check('tan(pi / 4)', tan(v(pi / 4)), 1);
  check('tan(-pi / 4)', tan(v(-pi / 4)), -1);
  check('tan(pi)', tan(v(pi)), 0);

  // acos
  check('acos(1)', acos(v(1)), 0);
  check('acos(0)', acos(v(0)), pi / 2);
  check('acos(-1)', acos(v(-1)), pi);
  check('acos(0.5)', acos(v(0.5)), pi / 3);
  checkNaN('acos(2)', acos(v(2)));

  // asin
  check('asin(0)', asin(v(0)), 0);
  check('asin(1)', asin(v(1)), pi / 2);
  check('asin(-1)', asin(v(-1)), -pi / 2);
  check('asin(0.5)', asin(v(0.5)), pi / 6);
  checkNaN('asin(2)', asin(v(2)));

  // atan
  check('atan(0)', atan(v(0)), 0);
  check('atan(1)', atan(v(1)), pi / 4);
  check('atan(-1)', atan(v(-1)), -pi / 4);
  check('atan(infinity)', atan(v(double.infinity)), pi / 2);

  // exp
  check('exp(0)', exp(v(0)), 1);
  check('exp(1)', exp(v(1)), e);
  check('exp(ln2)', exp(v(ln2)), 2);
  check('exp(-infinity)', exp(v(double.negativeInfinity)), 0);
  print('exp(infinity): ${exp(v(double.infinity))}');

  // log
  check('log(1)', log(v(1)), 0);
  check('log(e)', log(v(e)), 1);
  check('log(2)', log(v(2)), ln2);
  check('log(10)', log(v(10)), ln10);
  print('log(0): ${log(v(0))}');
  checkNaN('log(-1)', log(v(-1)));
}
