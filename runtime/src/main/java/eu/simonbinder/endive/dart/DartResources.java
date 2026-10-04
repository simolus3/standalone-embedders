package eu.simonbinder.endive.dart;

import java.net.URI;
import java.nio.file.Paths;
import java.security.SecureRandom;
import java.util.Random;
import java.util.function.Consumer;
import java.util.random.RandomGenerator;

public final class DartResources {
  private final Consumer<String> print;
  private final RandomGenerator insecureRandom;
  private final RandomGenerator secureRandom;
  private final EventLoop eventLoop;
  private final URI baseUri;
  private final boolean isWindows;

  DartResources(
      Consumer<String> print,
      RandomGenerator insecureRandom,
      RandomGenerator secureRandom,
      EventLoop eventLoop,
      URI baseUri,
      boolean isWindows) {
    this.print = print;
    this.insecureRandom = insecureRandom;
    this.secureRandom = secureRandom;
    this.eventLoop = eventLoop;
    this.baseUri = baseUri;
    this.isWindows = isWindows;
  }

  public RandomGenerator insecureRandom() {
    return insecureRandom;
  }

  public RandomGenerator secureRandom() {
    return secureRandom;
  }

  public URI baseUri() {
    return baseUri;
  }

  public boolean isWindows() {
    return isWindows;
  }

  public void print(String message) {
    print.accept(message);
  }

  public EventLoop eventLoop() {
    return eventLoop;
  }

  public static class Builder {
    private Consumer<String> print = IO::println;
    private RandomGenerator insecureRandom;
    private RandomGenerator secureRandom;
    private EventLoop eventLoop = EventLoop.UNSUPPORTED;
    private URI baseUri;
    private boolean isWindows = System.getProperty("os.name").toLowerCase().startsWith("win");

    public Builder print(Consumer<String> print) {
      this.print = print;
      return this;
    }

    public Builder baseUri(URI baseUri, boolean isWindows) {
      this.baseUri = baseUri;
      this.isWindows = isWindows;
      return this;
    }

    public Builder random(RandomGenerator insecure, RandomGenerator secure) {
      this.insecureRandom = insecure;
      this.secureRandom = secure;
      return this;
    }

    public Builder eventLoop(EventLoop eventLoop) {
      this.eventLoop = eventLoop;
      return this;
    }

    public DartResources build() {
      if (insecureRandom == null) {
        insecureRandom = new Random();
      }
      if (secureRandom == null) {
        secureRandom = new SecureRandom();
      }
      if (baseUri == null) {
        baseUri = Paths.get("").toAbsolutePath().toUri();
      }

      return new DartResources(print, insecureRandom, secureRandom, eventLoop, baseUri, isWindows);
    }
  }
}
