void main() {
  final sw = Stopwatch()..start();
  const expectedFrequency = bool.fromEnvironment('dart.tool.dart2wasm')
      ? 1_000_000
      : 1_000_000_000;

  print(sw.frequency == expectedFrequency);
  sw.elapsedTicks;
  sw.stop();
}
