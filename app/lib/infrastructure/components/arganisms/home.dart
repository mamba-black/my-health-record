import 'package:flutter/widgets.dart';
import 'package:flutter/material.dart';

class Home extends StatelessWidget {
  final int _counter;

  Home(this._counter, {super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: implement build
    return Column(children: [
      Container(
        alignment: Alignment.center,

        padding: const EdgeInsets.only(top: 40, bottom: 40),
        child: const Text(
          'Busqueda de paciente',
          style: TextStyle(fontSize: 30, fontWeight: FontWeight.bold),
        ),
      ),
      Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            const Text(
              'You have pushed the button this many times (2):',
            ),
            Text(
              '$_counter',
              style: Theme
                  .of(context)
                  .textTheme
                  .headline4,
            ),
          ],
        ),
      ),
    ]);
  }

}
