// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'incident.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$IncidentImpl _$$IncidentImplFromJson(Map<String, dynamic> json) =>
    _$IncidentImpl(
      id: json['id'] as String,
      timestamp: DateTime.parse(json['timestamp'] as String),
      severity: json['severity'] as String,
      chain:
          (json['chain'] as List<dynamic>?)?.map((e) => e as String).toList() ??
              const [],
      entities: (json['entities'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
      sigmaScore: (json['sigmaScore'] as num?)?.toDouble() ?? 0.0,
      zScore: (json['zScore'] as num?)?.toDouble() ?? 0.0,
      iocMatch: json['iocMatch'] as String?,
      cvss: (json['cvss'] as num?)?.toDouble() ?? 0.0,
      summary: json['summary'] as String?,
      actions:
          (json['actions'] as List<dynamic>?)?.map((e) => e as String).toList(),
    );

Map<String, dynamic> _$$IncidentImplToJson(_$IncidentImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'timestamp': instance.timestamp.toIso8601String(),
      'severity': instance.severity,
      'chain': instance.chain,
      'entities': instance.entities,
      'sigmaScore': instance.sigmaScore,
      'zScore': instance.zScore,
      'iocMatch': instance.iocMatch,
      'cvss': instance.cvss,
      'summary': instance.summary,
      'actions': instance.actions,
    };
