import 'dart:async';

void main() {
  var counter = 0;

  Timer.periodic(const Duration(seconds: 2), (timer) {
    if (++counter == 5) {
      timer.cancel();
    }

    print('Hello from Dart');
  });
}
