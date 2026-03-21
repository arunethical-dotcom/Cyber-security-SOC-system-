// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'feedback.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$UserFeedbackImpl _$$UserFeedbackImplFromJson(Map<String, dynamic> json) =>
    _$UserFeedbackImpl(
      incidentId: json['incidentId'] as String,
      action: json['action'] as String,
      entity: json['entity'] as String?,
      tactic: json['tactic'] as String?,
      newThreshold: (json['newThreshold'] as num?)?.toDouble(),
    );

Map<String, dynamic> _$$UserFeedbackImplToJson(_$UserFeedbackImpl instance) =>
    <String, dynamic>{
      'incidentId': instance.incidentId,
      'action': instance.action,
      'entity': instance.entity,
      'tactic': instance.tactic,
      'newThreshold': instance.newThreshold,
    };
