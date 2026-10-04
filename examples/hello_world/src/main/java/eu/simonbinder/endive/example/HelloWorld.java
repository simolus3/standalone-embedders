import static eu.simonbinder.endive.dart.DartWasmRuntime.EMPTY_ARGS;

import eu.simonbinder.demo.CompiledDartApp;
import eu.simonbinder.endive.dart.DartWasmRuntime;
import run.endive.runtime.Instance;
import run.endive.runtime.Store;
import run.endive.runtime.WasmArray;

void main() {
  var module = CompiledDartApp.load();
  var store = new Store();
  var runtime = new DartWasmRuntime(module);
  runtime.registerTo(store);

  var instance =
      Instance.builder(module)
          .withMachineFactory(CompiledDartApp::create)
          .withImportValues(store.toImportValues())
          .build();
  try (instance) {
    var main = instance.export("$invokeMain");
    var args = WasmArray.builder().build();

    main.applyWithRefs(EMPTY_ARGS, new Object[] {args});
  }
}
