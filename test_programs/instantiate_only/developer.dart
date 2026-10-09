import 'dart:developer';

void main() {
  debugger(message: 'foo');
  inspect(main);

  Timeline.startSync('bar');
  Timeline.finishSync();
}
