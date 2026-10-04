// ignore: import_internal_library
import 'dart:_wasm';

// Bindings to Swing, implemented in SwingCounter.java.

@pragma('wasm:import', 'swing.newFrame')
external WasmExternRef _newFrame(WasmExternRef title);

@pragma('wasm:import', 'swing.frameShow')
external WasmVoid _frameShow(WasmExternRef frame, WasmI32 width, WasmI32 height);

@pragma('wasm:import', 'swing.newLabel')
external WasmExternRef _newLabel(WasmExternRef text);

@pragma('wasm:import', 'swing.labelSetText')
external WasmVoid _labelSetText(WasmExternRef label, WasmExternRef text);

@pragma('wasm:import', 'swing.newButton')
external WasmExternRef _newButton(WasmExternRef text);

@pragma('wasm:import', 'swing.addActionListener')
external WasmVoid _addActionListener(
  WasmExternRef button,
  WasmFunction<WasmVoid Function(WasmAnyRef)> callback,
  WasmAnyRef arg,
);

@pragma('wasm:import', 'swing.setToolTipText')
external WasmVoid _setToolTipText(WasmExternRef component, WasmExternRef text);

@pragma('wasm:import', 'swing.setFont')
external WasmVoid _setFont(WasmExternRef component, WasmI32 style, WasmI32 size);

@pragma('wasm:import', 'swing.setEmptyBorder')
external WasmVoid _setEmptyBorder(
  WasmExternRef component,
  WasmI32 top,
  WasmI32 left,
  WasmI32 bottom,
  WasmI32 right,
);

@pragma('wasm:import', 'swing.newFlowPanel')
external WasmExternRef _newFlowPanel(WasmI32 alignment);

@pragma('wasm:import', 'swing.newGridBagPanel')
external WasmExternRef _newGridBagPanel();

@pragma('wasm:import', 'swing.add')
external WasmVoid _add(WasmExternRef container, WasmExternRef component);

@pragma('wasm:import', 'swing.addBorderLayout')
external WasmVoid _addBorderLayout(
  WasmExternRef container,
  WasmExternRef component,
  WasmExternRef position,
);

@pragma('wasm:import', 'swing.addGridBagColumn')
external WasmVoid _addGridBagColumn(
  WasmExternRef container,
  WasmExternRef component,
);

WasmExternRef _toJava(String value) =>
    WasmAnyRef.fromObject(value).externalize();

// Thin Dart wrappers around the Java objects.
//
// Note: Calls to imports returning `WasmVoid` must be statements. Returning
// them from a `void` arrow function makes dart2wasm emit `unreachable` after
// the call.

abstract final class Font {
  static const PLAIN = 0;
  static const BOLD = 1;
}

abstract final class FlowLayout {
  static const LEADING = 3;
  static const TRAILING = 4;
}

abstract final class BorderLayout {
  static const NORTH = 'North';
  static const CENTER = 'Center';
  static const SOUTH = 'South';
}

class JComponent {
  final WasmExternRef _ref;

  JComponent._(this._ref);

  void setToolTipText(String text) {
    _setToolTipText(_ref, _toJava(text));
  }

  void setFont(int style, int size) {
    _setFont(_ref, style.toWasmI32(), size.toWasmI32());
  }

  void setEmptyBorder(int top, int left, int bottom, int right) {
    _setEmptyBorder(
      _ref,
      top.toWasmI32(),
      left.toWasmI32(),
      bottom.toWasmI32(),
      right.toWasmI32(),
    );
  }
}

class JLabel extends JComponent {
  JLabel(String text) : super._(_newLabel(_toJava(text)));

  void setText(String text) {
    _labelSetText(_ref, _toJava(text));
  }
}

class JButton extends JComponent {
  JButton(String text) : super._(_newButton(_toJava(text)));

  void addActionListener(void Function() callback) {
    _addActionListener(
      _ref,
      WasmFunction.fromFunction(_invokeCallback),
      WasmAnyRef.fromObject(callback),
    );
  }

  static WasmVoid _invokeCallback(WasmAnyRef callback) {
    (callback.toObject() as void Function())();
    return WasmVoid();
  }
}

class JPanel extends JComponent {
  JPanel.flow(int alignment) : super._(_newFlowPanel(alignment.toWasmI32()));

  JPanel.gridBag() : super._(_newGridBagPanel());

  void add(JComponent component) {
    _add(_ref, component._ref);
  }

  void addColumn(JComponent component) {
    _addGridBagColumn(_ref, component._ref);
  }
}

class JFrame {
  final WasmExternRef _ref;

  JFrame(String title) : _ref = _newFrame(_toJava(title));

  void add(JComponent component, String position) {
    _addBorderLayout(_ref, component._ref, _toJava(position));
  }

  void show(int width, int height) {
    _frameShow(_ref, width.toWasmI32(), height.toWasmI32());
  }
}

// The counter app.

void main() {
  var counter = 0;
  final frame = JFrame('Dart Demo');

  final title = JLabel('WebAssembly Demo Home Page')..setFont(Font.PLAIN, 20);
  final appBar = JPanel.flow(FlowLayout.LEADING)
    ..setEmptyBorder(8, 8, 8, 8)
    ..add(title);

  final counterLabel = JLabel('$counter')..setFont(Font.PLAIN, 36);
  final body = JPanel.gridBag()
    ..addColumn(JLabel('You have pushed the button this many times:'))
    ..addColumn(counterLabel);

  final increment = JButton('+')
    ..setToolTipText('Increment')
    ..setFont(Font.BOLD, 24)
    ..addActionListener(() {
      counter++;
      counterLabel.setText('$counter');
    });
  final bottom = JPanel.flow(FlowLayout.TRAILING)
    ..setEmptyBorder(16, 16, 16, 16)
    ..add(increment);

  frame
    ..add(appBar, BorderLayout.NORTH)
    ..add(body, BorderLayout.CENTER)
    ..add(bottom, BorderLayout.SOUTH)
    ..show(400, 600);
}
