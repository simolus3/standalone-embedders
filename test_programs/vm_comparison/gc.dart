void main() {
  final target = _Target();
  final ref = WeakReference(target);
  print(ref.target);

  final expando = Expando<String>('test');
  final other = _Target();
  print(expando[target]);
  expando[target] = 'a';
  expando[other] = 'b';
  print(expando[target]);
  print(expando[other]);
  expando[target] = 'c';
  print(expando[target]);
  expando[target] = null;
  print(expando[target]);
  print(expando[other]);
  print(Expando<String>()[other]);

  // Many keys, to exercise distinct objects with potentially colliding hashes.
  final keys = [for (var i = 0; i < 1000; i++) _Target()];
  final numbers = Expando<int>();
  for (final (i, key) in keys.indexed) {
    numbers[key] = i;
  }
  var sum = 0;
  for (final key in keys) {
    sum += numbers[key]!;
  }
  print(sum);

  final finalizer = Finalizer<String>((token) => print('finalized $token'));
  final detachToken = Object();
  finalizer.attach(target, 'first');
  finalizer.attach(other, 'second', detach: detachToken);
  finalizer.detach(detachToken);
  print('attached');
}

final class _Target {
  @override
  String toString() {
    return 'target';
  }
}
