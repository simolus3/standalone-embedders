void main() async {
  final sw = Stopwatch()..start();
  const expectedFrequency = bool.fromEnvironment('dart.tool.dart2wasm')
      ? 1_000_000
      : 1_000_000_000;

  print(sw.frequency == expectedFrequency);

  const duration = Duration(microseconds: 50);
  await Future.pause(duration);

  sw.elapsedTicks;
  sw.stop();
  print(sw.elapsed >= duration);
}
