import 'package:freezed_annotation/freezed_annotation.dart';

part 'feedback.freezed.dart';
part 'feedback.g.dart';

@freezed
class UserFeedback with _$UserFeedback {
  const factory UserFeedback({
    required String incidentId,
    required String action,
    String? entity,
    String? tactic,
    double? newThreshold,
  }) = _UserFeedback;

  factory UserFeedback.fromJson(Map<String, dynamic> json) =>
      _$UserFeedbackFromJson(json);
}
