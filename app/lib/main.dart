import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';

import 'infrastructure/api/medical.pbgrpc.dart';
import 'infrastructure/components/arganisms/search_page.dart';

void main() {

  final channel = ClientChannel(
    'localhost',
    port: 50051,
    options: ChannelOptions(
      credentials: ChannelCredentials.insecure(),
      codecRegistry:
      CodecRegistry(codecs: const [GzipCodec(), IdentityCodec()]),
    ),
  );
  final client = HistoryClient(channel);

  runApp(MainApp(client));
}

class MainApp extends StatelessWidget {
  final HistoryClient client;
  const MainApp(this.client, {super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: SearchPage(title: 'Historias Clinicas', client: this.client),
    );
  }
}

