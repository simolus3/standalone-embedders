package eu.simonbinder.endive.example;

import static eu.simonbinder.endive.dart.DartWasmRuntime.EMPTY_ARGS;
import static eu.simonbinder.endive.dart.DartWasmRuntime.VOID_RETURN;
import static eu.simonbinder.endive.dart.DartWasmRuntime.dartCallback;
import static eu.simonbinder.endive.dart.DartWasmRuntime.dartString;

import eu.simonbinder.demo.CompiledDartApp;
import eu.simonbinder.endive.dart.DartWasmRuntime;
import java.awt.Container;
import java.awt.FlowLayout;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import javax.swing.BorderFactory;
import javax.swing.JButton;
import javax.swing.JComponent;
import javax.swing.JFrame;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.SwingUtilities;
import javax.swing.WindowConstants;
import run.endive.runtime.*;

/**
 * Launches the counter app implemented in {@code counter.dart}, providing the Swing bindings it
 * imports.
 */
public final class SwingCounter {
  private static void defineSwingBindings(DartWasmRuntime runtime) {
    runtime.defineExtraFunction(
        MODULE, "newFrame", (_, refs) -> object(new JFrame(dartString(refs[0]))));
    runtime.defineExtraFunction(
        MODULE,
        "frameShow",
        (args, refs) -> {
          var frame = (JFrame) refs[0];
          frame.setDefaultCloseOperation(WindowConstants.EXIT_ON_CLOSE);
          frame.setSize((int) args[1], (int) args[2]);
          frame.setLocationRelativeTo(null);
          frame.setVisible(true);
          return VOID_RETURN;
        });

    runtime.defineExtraFunction(
        MODULE, "newLabel", (_, refs) -> object(new JLabel(dartString(refs[0]))));
    runtime.defineExtraFunction(
        MODULE,
        "labelSetText",
        (_, refs) -> {
          ((JLabel) refs[0]).setText(dartString(refs[1]));
          return VOID_RETURN;
        });

    runtime.defineExtraFunction(
        MODULE, "newButton", (_, refs) -> object(new JButton(dartString(refs[0]))));
    runtime.defineExtraFunction(
        MODULE,
        "addActionListener",
        (instance, args, refs) -> {
          var callback = dartCallback(instance, args[1]);
          var arg = refs[2];
          ((JButton) refs[0]).addActionListener(_ -> callback.accept(arg));
          return VOID_RETURN;
        });

    runtime.defineExtraFunction(
        MODULE,
        "setToolTipText",
        (_, refs) -> {
          ((JComponent) refs[0]).setToolTipText(dartString(refs[1]));
          return VOID_RETURN;
        });
    runtime.defineExtraFunction(
        MODULE,
        "setFont",
        (args, refs) -> {
          var component = (JComponent) refs[0];
          component.setFont(component.getFont().deriveFont((int) args[1], (float) args[2]));
          return VOID_RETURN;
        });
    runtime.defineExtraFunction(
        MODULE,
        "setEmptyBorder",
        (args, refs) -> {
          ((JComponent) refs[0])
              .setBorder(
                  BorderFactory.createEmptyBorder(
                      (int) args[1], (int) args[2], (int) args[3], (int) args[4]));
          return VOID_RETURN;
        });

    runtime.defineExtraFunction(
        MODULE, "newFlowPanel", (args, _) -> object(new JPanel(new FlowLayout((int) args[0]))));
    runtime.defineExtraFunction(
        MODULE, "newGridBagPanel", (_, _) -> object(new JPanel(new GridBagLayout())));
    runtime.defineExtraFunction(
        MODULE,
        "add",
        (_, refs) -> {
          ((Container) refs[0]).add((JComponent) refs[1]);
          return VOID_RETURN;
        });
    runtime.defineExtraFunction(
        MODULE,
        "addBorderLayout",
        (_, refs) -> {
          ((Container) refs[0]).add((JComponent) refs[1], dartString(refs[2]));
          return VOID_RETURN;
        });
    runtime.defineExtraFunction(
        MODULE,
        "addGridBagColumn",
        (_, refs) -> {
          var constraints = new GridBagConstraints();
          constraints.gridx = 0;
          ((Container) refs[0]).add((JComponent) refs[1], constraints);
          return VOID_RETURN;
        });
  }

  private static CallResult object(Object result) {
    return CallResult.of(EMPTY_ARGS, new Object[] {result});
  }

  public static void main(String[] args) {
    var module = CompiledDartApp.load();
    var store = new Store();
    var runtime = new DartWasmRuntime(module);
    defineSwingBindings(runtime);
    runtime.registerTo(store);

    var instance =
        Instance.builder(module)
            .withMachineFactory(CompiledDartApp::create)
            .withImportValues(store.toImportValues())
            .build();

    // The Dart app builds its UI in main, so run it on the event dispatch thread. Callbacks from
    // Swing are dispatched on that thread too, so the instance is only used by a single thread.
    SwingUtilities.invokeLater(
        () -> {
          var main = instance.export("$invokeMain");
          var emptyArgs = WasmArray.builder().build();
          main.applyWithRefs(EMPTY_ARGS, new Object[] {emptyArgs});
        });
  }

  private static final String MODULE = "swing";
}
