import 'package:freezed_annotation/freezed_annotation.dart';

part 'incident.freezed.dart';
part 'incident.g.dart';

@freezed
class Incident with _$Incident {
  const factory Incident({
    required String id,
    required DateTime timestamp,
    required String severity,
    @Default([]) List<String> chain,
    @Default([]) List<String> entities,
    @Default(0.0) double sigmaScore,
    @Default(0.0) double zScore,
    String? iocMatch,
    @Default(0.0) double cvss,
    String? summary,
    List<String>? actions,
  }) = _Incident;

  factory Incident.fromJson(Map<String, dynamic> json) =>
      _$IncidentFromJson(json);
}
