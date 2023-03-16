///
//  Generated code. Do not modify.
//  source: medical.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,constant_identifier_names,directives_ordering,library_prefixes,non_constant_identifier_names,prefer_final_fields,return_of_invalid_type,unnecessary_const,unnecessary_import,unnecessary_this,unused_import,unused_shown_name

import 'dart:async' as $async;

import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'medical.pb.dart' as $0;
export 'medical.pb.dart';

class HistoryClient extends $grpc.Client {
  static final _$find =
      $grpc.ClientMethod<$0.PatientRequest, $0.PatientResponse>(
          '/infrastructure.api.History/Find',
          ($0.PatientRequest value) => value.writeToBuffer(),
          ($core.List<$core.int> value) =>
              $0.PatientResponse.fromBuffer(value));

  HistoryClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);

  $grpc.ResponseFuture<$0.PatientResponse> find($0.PatientRequest request,
      {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$find, request, options: options);
  }
}

abstract class HistoryServiceBase extends $grpc.Service {
  $core.String get $name => 'infrastructure.api.History';

  HistoryServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.PatientRequest, $0.PatientResponse>(
        'Find',
        find_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PatientRequest.fromBuffer(value),
        ($0.PatientResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.PatientResponse> find_Pre(
      $grpc.ServiceCall call, $async.Future<$0.PatientRequest> request) async {
    return find(call, await request);
  }

  $async.Future<$0.PatientResponse> find(
      $grpc.ServiceCall call, $0.PatientRequest request);
}
