import 'package:flutter/material.dart';
import 'dart:developer';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        // This is the theme of your application.
        //
        // Try running your application with "flutter run". You'll see the
        // application has a blue toolbar. Then, without quitting the app, try
        // changing the primarySwatch below to Colors.green and then invoke
        // "hot reload" (press "r" in the console where you ran "flutter run",
        // or simply save your changes to "hot reload" in a Flutter IDE).
        // Notice that the counter didn't reset back to zero; the application
        // is not restarted.
        primarySwatch: Colors.blue,
      ),
      home: const MyHomePage(title: 'Historias Clinicas'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  final String title;

  const MyHomePage({super.key, required this.title});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  int _counter = 0;
  List<String> _results = [];

  void _incrementCounter() {
    setState(() {
      _counter++;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.title),
        centerTitle: MediaQuery.of(context).size.width < 600 ? true : false,
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () {
              showSearch(context: context, delegate: CustomSearchDelegate(_results));
            },
          ),
        ],
      ),
      body: Column(children: [
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
                'You have pushed the button this many times (1):',
              ),
              Text(
                '$_counter',
                style: Theme.of(context).textTheme.headline4,
              ),
            ],
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
                style: Theme.of(context).textTheme.headline4,
              ),
            ],
          ),
        ),
      ]),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}

class CustomSearchDelegate extends SearchDelegate {
  List<String> _results;

  CustomSearchDelegate(this._results) {
  }


  @override
  List<Widget>? buildActions(BuildContext context) {
    return [
      IconButton(
          onPressed: () {
            query = '';
            _results = [];
          },
          icon: const Icon(Icons.clear))
    ];
  }

  @override
  Widget? buildLeading(BuildContext context) {
    return IconButton(
        onPressed: () {
          close(context, null);
        },
        icon: const Icon(Icons.arrow_back));
  }

  @override
  Widget buildResults(BuildContext context) {
    log('buildResults $context');
    _results = ["uno", "dos", "tres", DateTime.now().toString()];

    return ListView.builder(
      itemCount: _results.length,
      itemBuilder: (context, index) {
        var result = _results[index];
        var avatar = Image.network('https://en.gravatar.com/userimage/23111536/064cda8270d684ed4f0824fc8cba267b.jpeg');
        var listTile = ListTile(
          title: Text('Result $index: $result'),
          subtitle: Text('Result $index: $result'),
          // leading: Image.network('https://en.gravatar.com/userimage/23111536/064cda8270d684ed4f0824fc8cba267b.jpeg'),
          leading: ClipOval(
            child: avatar,
          ),
          onTap: () {
            log('onTap $index');
            // query = 'Otro valor';
          },
        );
        return listTile;
      },
    );
  }

  @override
  Widget buildSuggestions(BuildContext context) {
    log('buildResults $context');
    _results = ["Uno", "Dos", "Tres", DateTime.now().toString()];

    return ListView.builder(
      itemCount: _results.length,
      itemBuilder: (context, index) {
        var result = _results[index];
        var avatar = Image.network('https://en.gravatar.com/userimage/23111536/064cda8270d684ed4f0824fc8cba267b.jpeg');
        var listTile = ListTile(
          title: Text('Result $index: $result'),
          subtitle: Text('Result $index: $result'),
          // leading: Image.network('https://en.gravatar.com/userimage/23111536/064cda8270d684ed4f0824fc8cba267b.jpeg'),
          leading: ClipOval(
            child: avatar,
          ),
          onTap: () {
            log('onTap $index');
            // query = 'Otro valor';
          },
        );
        return listTile;
      },
    );
  }
}
