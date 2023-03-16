import 'package:flutter/material.dart';

class Field extends SizedBox {
  // String labelText;
  // TextEditingController? controller;
  // double? _width;
  // double? min;

  Field(String labelText, TextEditingController? controller, [double? max, double? _width, double? min])
      : super(
          width: (_width != null && min != null && _width < min) ? max : _width,
          child: TextFormField(
            decoration: InputDecoration(border: const OutlineInputBorder(), labelText: labelText),
            controller: controller,
          ),
        );

}
