import 'dart:async';

void main() {
  final a = Timer.periodic(const Duration(milliseconds: 50), (_) {
    print('50ms tick');
  });

  Timer(const Duration(milliseconds: 50), a.cancel);
  print('set timers');
}
