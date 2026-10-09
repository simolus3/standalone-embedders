void main() {
  final target = _Target();
  final ref = WeakReference(target);
  print(ref.target);
}

final class _Target {
  @override
  String toString() {
    return 'target';
  }
}
