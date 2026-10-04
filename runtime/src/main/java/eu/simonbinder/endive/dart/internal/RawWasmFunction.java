package eu.simonbinder.endive.dart.internal;

import run.endive.runtime.CallResult;
import run.endive.runtime.Instance;
import run.endive.runtime.WasmFunctionHandle;

@FunctionalInterface
public interface RawWasmFunction extends WasmFunctionHandle {
  @Override
  default long[] apply(Instance instance, long... args) {
    throw new IllegalArgumentException("Call applyWithRefs instead");
  }

  CallResult applyWithRefs(Instance instance, long[] args, Object[] refArgs);
}
