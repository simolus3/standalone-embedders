package eu.simonbinder.endive.dart;

import eu.simonbinder.endive.dart.internal.RawWasmFunction;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.lang.ref.WeakReference;
import java.time.Duration;
import java.util.*;
import java.util.function.BiFunction;
import java.util.function.Consumer;
import java.util.function.DoubleBinaryOperator;
import java.util.function.DoubleUnaryOperator;
import run.endive.runtime.*;
import run.endive.wasm.WasmModule;
import run.endive.wasm.types.FunctionImport;
import run.endive.wasm.types.FunctionType;
import run.endive.wasm.types.ValType;

public final class DartWasmRuntime {
  private final DartResources resources;

  private final List<HostFunction> functions = new ArrayList<>();
  private final Map<String, FunctionType> importedFunctions = new HashMap<>();

  public DartWasmRuntime(WasmModule module) {
    this(module, new DartResources.Builder().build());
  }

  public DartWasmRuntime(WasmModule module, DartResources resources) {
    this.resources = resources;
    module.importSection().stream()
        .forEach(
            imported -> {
              if (imported instanceof FunctionImport fn) {
                var type = module.typeSection().getType(fn.typeIndex());
                importedFunctions.put(fn.name(), type);
              }
            });

    definePrint();
    defineString();
    defineStringBuffer();
    defineClock();
    defineCoreMisc();
    defineDeveloper();
    defineMath();

    defineFunction(
        "jsonEncodeString",
        (_, _) -> {
          throw new IllegalStateException("TODO");
        });
  }

  private void defineRawFunction(String name, WasmFunctionHandle function) {
    var type = importedFunctions.get(name);
    if (type == null) {
      return;
    }

    functions.add(new HostFunction(NAMESPACE, name, type, function));
  }

  /** A host function that has access to the calling {@link Instance}. */
  @FunctionalInterface
  public interface ExtraFunction {
    CallResult apply(Instance instance, long[] args, Object[] refArgs);
  }

  public void defineExtraFunction(
      String module, String name, BiFunction<long[], Object[], CallResult> function) {
    defineExtraFunction(module, name, (_, args, refArgs) -> function.apply(args, refArgs));
  }

  public void defineExtraFunction(String module, String name, ExtraFunction function) {
    var type = importedFunctions.get(name);
    if (type == null) {
      return;
    }

    functions.add(
        new HostFunction(
            module,
            name,
            type,
            new WasmFunctionHandle() {
              @Override
              public long[] apply(Instance instance, long... args) {
                return null;
              }

              @Override
              public CallResult applyWithRefs(Instance instance, long[] args, Object[] refArgs) {
                return function.apply(instance, args, refArgs);
              }
            }));
  }

  private void defineFunction(String name, BiFunction<long[], Object[], Object> function) {
    defineRawFunction(name, wasmFunction(function));
  }

  private void defineUnaryDoubleFunction(String name, DoubleUnaryOperator function) {
    var type = importedFunctions.get(name);
    if (type == null) {
      return;
    }

    functions.add(
        new HostFunction(
            NAMESPACE,
            name,
            type,
            (instance, args) -> {
              var arg = Double.longBitsToDouble(args[0]);
              var result = function.applyAsDouble(arg);
              return new long[] {Double.doubleToLongBits(result)};
            }));
  }

  private void defineBinaryDoubleFunction(String name, DoubleBinaryOperator function) {
    var type = importedFunctions.get(name);
    if (type == null) {
      return;
    }

    functions.add(
        new HostFunction(
            NAMESPACE,
            name,
            type,
            (instance, args) -> {
              var arg0 = Double.longBitsToDouble(args[0]);
              var arg1 = Double.longBitsToDouble(args[1]);
              var result = function.applyAsDouble(arg0, arg1);
              return new long[] {Double.doubleToLongBits(result)};
            }));
  }

  private void defineString() {
    defineFunction(
        "stringFromAsciiBytes",
        (var numeric, var refs) -> {
          var asciiBytes = (WasmArray) refs[0];
          var start = (int) numeric[1];
          var length = (int) numeric[2];

          var buffer = new StringBuilder(length);
          for (int i = 0; i < length; i++) {
            buffer.append((char) asciiBytes.get(start + i));
          }

          return buffer.toString();
        });
    defineFunction("stringLength", (_, args) -> ((String) args[0]).length());
    defineFunction("i64ToString", (args, _) -> Long.toString(args[0]));
    // TODO: f64ToExponential
    // TODO: f64ToExponentialWithFractionDigits
    // TODO: f64ToPrecision
    // TODO: f64ToFixed
    defineFunction("f64ToString", (args, _) -> Double.toString(Double.longBitsToDouble(args[0])));

    defineFunction(
        "doubleParseInfallible", (_, args) -> doubleResult(Double.parseDouble((String) args[0])));
    defineFunction(
        "doubleTryParse",
        (_, args) -> {
          try {
            return Double.parseDouble((String) args[0]);
          } catch (NumberFormatException e) {
            return null;
          }
        });
    defineFunction("tryParseResultGetDouble", (_, args) -> doubleResult((double) args[0]));
  }

  private void defineStringBuffer() {
    defineFunction("stringBufferCreate", (_, _) -> new StringBuilder());
    defineFunction(
        "stringBufferWriteString",
        (_, args) -> {
          ((StringBuilder) args[0]).append((String) args[1]);
          return null;
        });
    defineFunction(
        "stringBufferWriteCharCode",
        (_, args) -> {
          ((StringBuilder) args[0]).append((char) args[1]);
          return null;
        });
    defineFunction(
        "stringBufferClear",
        (_, args) -> {
          ((StringBuilder) args[0]).setLength(0);
          return null;
        });
    defineFunction("stringBufferLength", (_, args) -> ((StringBuilder) args[0]).length());
    defineFunction("stringBufferToString", (_, args) -> ((StringBuilder) args[0]).toString());
  }

  private void defineCoreMisc() {
    defineFunction(
        "stackTraceGetCurrent",
        (_, _) -> {
          var writer = new StringWriter();
          var printWriter = new PrintWriter(writer);
          new Exception().printStackTrace(printWriter);
          printWriter.close();
          return writer.toString();
        });
    defineFunction("stackTraceToString", (_, args) -> args[0]);

    defineFunction("baseUri", (_, _) -> resources.baseUri().toString());
    defineFunction("isWindows", (_, _) -> longResult(resources.isWindows() ? 1 : 0));

    defineFunction("weakRefCreate", (_, args) -> new WeakReference<>(args[0]));
    defineFunction("weakRefGet", (_, args) -> ((WeakReference<?>) args[0]).get());
    defineFunction("expandoCreate", (_, _) -> new WeakHashMap<>());
    defineFunction("expandoGet", (_, args) -> ((WeakHashMap<?, ?>) args[0]).get(args[1]));
    //noinspection unchecked
    defineFunction(
        "expandoSet", (_, args) -> ((WeakHashMap<Object, Object>) args[0]).put(args[1], args[3]));

    // finalizerCreate
    // finalizerAttach
    // finalizerDetach
  }

  private void defineClock() {
    defineRawFunction(
        "queueMicrotask",
        rawWasmFunction(
            (instance, args, refArgs) -> {
              var function = extractUnaryFunction(instance, args[0]);
              var arg = refArgs[1];

              resources.eventLoop().queueMicrotask(function, arg);
              return CallResult.of(EMPTY_ARGS, new Object[0]);
            }));
    defineRawFunction(
        "scheduleOnce",
        rawWasmFunction(
            (instance, args, refArgs) -> {
              var delay = Duration.ofNanos(args[0] * 1000);
              var function = extractUnaryFunction(instance, args[1]);
              var arg = refArgs[2];

              var timer = resources.eventLoop().scheduleOnce(delay, function, arg);
              return CallResult.of(EMPTY_ARGS, new Object[] {timer});
            }));
    defineRawFunction(
        "scheduleRepeated",
        rawWasmFunction(
            (instance, args, refArgs) -> {
              var interval = Duration.ofNanos(args[0] * 1000);
              var function = extractUnaryFunction(instance, args[1]);
              var arg = refArgs[2];

              var timer = resources.eventLoop().scheduleRepeated(interval, function, arg);
              return CallResult.of(EMPTY_ARGS, new Object[] {timer});
            }));
    defineFunction(
        "clearSchedule",
        (_, args) -> {
          ((EventLoop.Timer) args[0]).clear();
          return null;
        });

    // monotonicClockFrequency
    // monotonicClockTicks
    // currentTimeMicros
    // timeZoneNameForClampedSeconds
    // timeZoneOffsetInSecondsForClampedSeconds
  }

  private void definePrint() {
    defineFunction(
        "print",
        (_, args) -> {
          resources.print((String) args[0]);
          return null;
        });
  }

  private void defineDeveloper() {
    defineFunction("debugger", (_, _) -> null);
    defineFunction("inspect", (_, _) -> null);
    defineFunction(
        "dartTimelineStreamEnabled", (_, _) -> CallResult.of(new long[] {0}, new Object[] {}));
    defineFunction("reportTaskEvent", (_, _) -> CallResult.of(new long[] {0}, new Object[] {}));
  }

  private void defineMath() {
    defineBinaryDoubleFunction("mathPow", Math::pow);
    defineBinaryDoubleFunction("mathAtan2", Math::atan2);
    defineUnaryDoubleFunction("mathSin", Math::sin);
    defineUnaryDoubleFunction("mathCos", Math::cos);
    defineUnaryDoubleFunction("mathTan", Math::tan);
    defineUnaryDoubleFunction("mathAcos", Math::acos);
    defineUnaryDoubleFunction("mathAsin", Math::asin);
    defineUnaryDoubleFunction("mathAtan", Math::atan);
    defineUnaryDoubleFunction("mathExp", Math::exp);
    defineUnaryDoubleFunction("mathLog", Math::log);

    defineFunction(
        "randomInt",
        (_, _) -> {
          var result = resources.insecureRandom().nextLong();
          return CallResult.of(new long[] {result}, new Object[] {});
        });
    defineFunction(
        "randomIntSecure",
        (_, _) -> {
          var result = resources.secureRandom().nextLong();
          return CallResult.of(new long[] {result}, new Object[] {});
        });
  }

  public void registerTo(Store store) {
    for (var function : functions) {
      store.addFunction(function);
    }
  }

  private static final String NAMESPACE = "dart";
  private static final ValType[] VOID = new ValType[] {};
  public static final long[] EMPTY_ARGS = new long[0];

  private static RawWasmFunction wasmFunction(BiFunction<long[], Object[], Object> function) {
    return (_, args, refArgs) -> {
      var result = function.apply(args, refArgs);
      if (result instanceof CallResult rs) {
        return rs;
      }

      return CallResult.of(EMPTY_ARGS, new Object[] {result});
    };
  }

  private static RawWasmFunction rawWasmFunction(RawWasmFunction function) {
    return function;
  }

  private static CallResult doubleResult(double result) {
    return longResult(Double.doubleToLongBits(result));
  }

  private static CallResult longResult(long result) {
    return CallResult.of(new long[] {result}, new Object[] {});
  }

  private static Consumer<Object> extractUnaryFunction(Instance instance, long index) {
    return arg -> instance.getMachine().callWithRefs((int) index, EMPTY_ARGS, new Object[] {arg});
  }

  /**
   * Returns a callback invoking a Dart function of type {@code WasmVoid Function(WasmAnyRef)},
   * passed to the host as a {@code WasmFunction}.
   */
  public static Consumer<Object> dartCallback(Instance instance, long functionIndex) {
    return extractUnaryFunction(instance, functionIndex);
  }

  /**
   * Extracts the Java string from a Dart {@code String} passed to the host with {@code
   * WasmAnyRef.fromObject(string).externalize()}.
   */
  public static String dartString(Object ref) {
    if (ref instanceof WasmStruct struct && struct.fieldRef(1) instanceof String string) {
      return string;
    }

    throw new IllegalArgumentException("Not a Dart string: " + ref);
  }

  public static final CallResult VOID_RETURN = CallResult.of(EMPTY_ARGS, new Object[] {});
}
