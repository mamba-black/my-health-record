import 'dart:collection';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:medical_app/infrastructure/components/arganisms/patient_information.dart';
import '../../api/medical.pbgrpc.dart';
import 'dart:developer';

import 'home.dart';

class SearchPage extends StatefulWidget {
  final String title;
  final HistoryClient client;

  const SearchPage({super.key, required this.title, required this.client});

  @override
  State<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends State<SearchPage> {
  final String _searchText = 'Busqueda de pacientes';
  int _counter = 0;
  final List<String> _results = [];
  final Queue<String> _namePatients = Queue<String>();
  String? resultSearch;

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
            tooltip: _searchText,
            onPressed: () async {
              final _resultSearch =
                  await showSearch<String?>(context: context, delegate: CustomSearchDelegate(_namePatients, _results));
              setState(() {
                resultSearch = _resultSearch;
              });
            },
          ),
        ],
      ),
      body: resultSearch == null ? Home(_counter) : PatientInformation(),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}

class CustomSearchDelegate extends SearchDelegate<String?> {
  static const int LIMIT_CACHE = 10;

  List<String> _results;
  Queue<String> _namePatients = Queue<String>();

  CustomSearchDelegate(this._namePatients, this._results);

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
        tooltip: 'Regresar',
        icon: const Icon(Icons.arrow_back));
  }

  @override
  Widget buildSuggestions(BuildContext context) {
    log('buildResults $context');

    Widget widget;

    if (query.isEmpty && _namePatients.isEmpty) {
      widget = const Center(
        child: Text('Escriba el nombre del paciente'),
      );
    } else if (query.trim().length < 4 && _namePatients.isEmpty) {
      widget = const Center(
        child: Text('El nombre debe contener al menos 4 caracteres'),
      );
    } else {
      widget = ListView.builder(
        itemCount: _namePatients.length,
        itemBuilder: (context, index) {
          var patientName = _namePatients.elementAt(index);
          var listTile = ListTile(
            title: Text(patientName),
            onTap: () {
              log('onTap $index');
              log('context $context');
              query = patientName;
              showResults(context);
              // query = 'Otro valor';
            },
          );
          return listTile;
        },
      );
    }

    return widget;
  }

  @override
  Widget buildResults(BuildContext context) {
    log('buildResults $context');
    Widget widget;

    if (query.trim().isNotEmpty) {
      widget = StreamBuilder<String>(
          stream: searchPatient(),
          builder: (context, snapshot) {
            if (!snapshot.hasData) {
              return const Center(
                child: Text('No se encontro el paciente'),
              );
            }

            if (snapshot.data != null) {
              _results.add(snapshot.data as String);
            }
            return ListView.builder(
              itemCount: _results.length,
              itemBuilder: (context, index) {
                var result = _results[index];
                var avatar =
                    Image.network('https://en.gravatar.com/userimage/23111536/064cda8270d684ed4f0824fc8cba267b.jpeg');
                var listTile = ListTile(
                  title: Text('Result $index: $result'),
                  subtitle: Text('Result $index: $result'),
                  // leading: Image.network('https://en.gravatar.com/userimage/23111536/064cda8270d684ed4f0824fc8cba267b.jpeg'),
                  leading: ClipOval(
                    child: avatar,
                  ),
                  onTap: () {
                    log('onTap $index, regresar al home');
                    close(context, "Se encontro al paciente");
                  },
                );
                return listTile;
              },
            );
          });
    } else {
      widget = const Center(child: Text('El nombre debe contener al menos 4 caracteres'));
    }

    return widget;
  }

  // SERVICIOS QUE DEBEN ESTAR EN UN SERVICE ==========================================================================>
  Stream<String> searchPatient() {
    // var stream = Stream.value("12345678");
    var stream = Stream<String>.periodic(const Duration(seconds: 1), (x) => x.toString()).take(15);
    _results = [];
    if (_namePatients.length > LIMIT_CACHE) {
      _namePatients.removeLast();
    }

    if (_namePatients.isEmpty && query.trim().isNotEmpty) {
      _namePatients.addFirst(query);
    } else if (_namePatients.isNotEmpty && query.trim().isNotEmpty && _namePatients.last != query) {
      _namePatients.addFirst(query);
    }

    return stream;
  }
// SERVICIOS QUE DEBEN ESTAR EN UN SERVICE ==========================================================================<

}
