package eu.simonbinder.endive.dart;

import java.time.Duration;
import java.util.function.Consumer;

public interface EventLoop {
  Timer scheduleOnce(Duration delay, Consumer<Object> callback, Object arg);

  Timer scheduleRepeated(Duration interval, Consumer<Object> callback, Object arg);

  void queueMicrotask(Consumer<Object> callback, Object arg);

  interface Timer {
    void clear();
  }

  EventLoop UNSUPPORTED =
      new EventLoop() {
        @Override
        public Timer scheduleOnce(Duration delay, Consumer<Object> callback, Object arg) {
          throw new UnsupportedOperationException();
        }

        @Override
        public Timer scheduleRepeated(Duration interval, Consumer<Object> callback, Object arg) {
          throw new UnsupportedOperationException();
        }

        @Override
        public void queueMicrotask(Consumer<Object> callback, Object arg) {
          throw new UnsupportedOperationException();
        }
      };
}
