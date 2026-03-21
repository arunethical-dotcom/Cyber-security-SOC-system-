// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'incident.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Incident _$IncidentFromJson(Map<String, dynamic> json) {
  return _Incident.fromJson(json);
}

/// @nodoc
mixin _$Incident {
  String get id => throw _privateConstructorUsedError;
  DateTime get timestamp => throw _privateConstructorUsedError;
  String get severity => throw _privateConstructorUsedError;
  List<String> get chain => throw _privateConstructorUsedError;
  List<String> get entities => throw _privateConstructorUsedError;
  double get sigmaScore => throw _privateConstructorUsedError;
  double get zScore => throw _privateConstructorUsedError;
  String? get iocMatch => throw _privateConstructorUsedError;
  double get cvss => throw _privateConstructorUsedError;
  String? get summary => throw _privateConstructorUsedError;
  List<String>? get actions => throw _privateConstructorUsedError;

  /// Serializes this Incident to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of Incident
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $IncidentCopyWith<Incident> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $IncidentCopyWith<$Res> {
  factory $IncidentCopyWith(Incident value, $Res Function(Incident) then) =
      _$IncidentCopyWithImpl<$Res, Incident>;
  @useResult
  $Res call(
      {String id,
      DateTime timestamp,
      String severity,
      List<String> chain,
      List<String> entities,
      double sigmaScore,
      double zScore,
      String? iocMatch,
      double cvss,
      String? summary,
      List<String>? actions});
}

/// @nodoc
class _$IncidentCopyWithImpl<$Res, $Val extends Incident>
    implements $IncidentCopyWith<$Res> {
  _$IncidentCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Incident
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? timestamp = null,
    Object? severity = null,
    Object? chain = null,
    Object? entities = null,
    Object? sigmaScore = null,
    Object? zScore = null,
    Object? iocMatch = freezed,
    Object? cvss = null,
    Object? summary = freezed,
    Object? actions = freezed,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as DateTime,
      severity: null == severity
          ? _value.severity
          : severity // ignore: cast_nullable_to_non_nullable
              as String,
      chain: null == chain
          ? _value.chain
          : chain // ignore: cast_nullable_to_non_nullable
              as List<String>,
      entities: null == entities
          ? _value.entities
          : entities // ignore: cast_nullable_to_non_nullable
              as List<String>,
      sigmaScore: null == sigmaScore
          ? _value.sigmaScore
          : sigmaScore // ignore: cast_nullable_to_non_nullable
              as double,
      zScore: null == zScore
          ? _value.zScore
          : zScore // ignore: cast_nullable_to_non_nullable
              as double,
      iocMatch: freezed == iocMatch
          ? _value.iocMatch
          : iocMatch // ignore: cast_nullable_to_non_nullable
              as String?,
      cvss: null == cvss
          ? _value.cvss
          : cvss // ignore: cast_nullable_to_non_nullable
              as double,
      summary: freezed == summary
          ? _value.summary
          : summary // ignore: cast_nullable_to_non_nullable
              as String?,
      actions: freezed == actions
          ? _value.actions
          : actions // ignore: cast_nullable_to_non_nullable
              as List<String>?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$IncidentImplCopyWith<$Res>
    implements $IncidentCopyWith<$Res> {
  factory _$$IncidentImplCopyWith(
          _$IncidentImpl value, $Res Function(_$IncidentImpl) then) =
      __$$IncidentImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      DateTime timestamp,
      String severity,
      List<String> chain,
      List<String> entities,
      double sigmaScore,
      double zScore,
      String? iocMatch,
      double cvss,
      String? summary,
      List<String>? actions});
}

/// @nodoc
class __$$IncidentImplCopyWithImpl<$Res>
    extends _$IncidentCopyWithImpl<$Res, _$IncidentImpl>
    implements _$$IncidentImplCopyWith<$Res> {
  __$$IncidentImplCopyWithImpl(
      _$IncidentImpl _value, $Res Function(_$IncidentImpl) _then)
      : super(_value, _then);

  /// Create a copy of Incident
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? timestamp = null,
    Object? severity = null,
    Object? chain = null,
    Object? entities = null,
    Object? sigmaScore = null,
    Object? zScore = null,
    Object? iocMatch = freezed,
    Object? cvss = null,
    Object? summary = freezed,
    Object? actions = freezed,
  }) {
    return _then(_$IncidentImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      timestamp: null == timestamp
          ? _value.timestamp
          : timestamp // ignore: cast_nullable_to_non_nullable
              as DateTime,
      severity: null == severity
          ? _value.severity
          : severity // ignore: cast_nullable_to_non_nullable
              as String,
      chain: null == chain
          ? _value._chain
          : chain // ignore: cast_nullable_to_non_nullable
              as List<String>,
      entities: null == entities
          ? _value._entities
          : entities // ignore: cast_nullable_to_non_nullable
              as List<String>,
      sigmaScore: null == sigmaScore
          ? _value.sigmaScore
          : sigmaScore // ignore: cast_nullable_to_non_nullable
              as double,
      zScore: null == zScore
          ? _value.zScore
          : zScore // ignore: cast_nullable_to_non_nullable
              as double,
      iocMatch: freezed == iocMatch
          ? _value.iocMatch
          : iocMatch // ignore: cast_nullable_to_non_nullable
              as String?,
      cvss: null == cvss
          ? _value.cvss
          : cvss // ignore: cast_nullable_to_non_nullable
              as double,
      summary: freezed == summary
          ? _value.summary
          : summary // ignore: cast_nullable_to_non_nullable
              as String?,
      actions: freezed == actions
          ? _value._actions
          : actions // ignore: cast_nullable_to_non_nullable
              as List<String>?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$IncidentImpl implements _Incident {
  const _$IncidentImpl(
      {required this.id,
      required this.timestamp,
      required this.severity,
      final List<String> chain = const [],
      final List<String> entities = const [],
      this.sigmaScore = 0.0,
      this.zScore = 0.0,
      this.iocMatch,
      this.cvss = 0.0,
      this.summary,
      final List<String>? actions})
      : _chain = chain,
        _entities = entities,
        _actions = actions;

  factory _$IncidentImpl.fromJson(Map<String, dynamic> json) =>
      _$$IncidentImplFromJson(json);

  @override
  final String id;
  @override
  final DateTime timestamp;
  @override
  final String severity;
  final List<String> _chain;
  @override
  @JsonKey()
  List<String> get chain {
    if (_chain is EqualUnmodifiableListView) return _chain;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_chain);
  }

  final List<String> _entities;
  @override
  @JsonKey()
  List<String> get entities {
    if (_entities is EqualUnmodifiableListView) return _entities;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_entities);
  }

  @override
  @JsonKey()
  final double sigmaScore;
  @override
  @JsonKey()
  final double zScore;
  @override
  final String? iocMatch;
  @override
  @JsonKey()
  final double cvss;
  @override
  final String? summary;
  final List<String>? _actions;
  @override
  List<String>? get actions {
    final value = _actions;
    if (value == null) return null;
    if (_actions is EqualUnmodifiableListView) return _actions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(value);
  }

  @override
  String toString() {
    return 'Incident(id: $id, timestamp: $timestamp, severity: $severity, chain: $chain, entities: $entities, sigmaScore: $sigmaScore, zScore: $zScore, iocMatch: $iocMatch, cvss: $cvss, summary: $summary, actions: $actions)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IncidentImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.timestamp, timestamp) ||
                other.timestamp == timestamp) &&
            (identical(other.severity, severity) ||
                other.severity == severity) &&
            const DeepCollectionEquality().equals(other._chain, _chain) &&
            const DeepCollectionEquality().equals(other._entities, _entities) &&
            (identical(other.sigmaScore, sigmaScore) ||
                other.sigmaScore == sigmaScore) &&
            (identical(other.zScore, zScore) || other.zScore == zScore) &&
            (identical(other.iocMatch, iocMatch) ||
                other.iocMatch == iocMatch) &&
            (identical(other.cvss, cvss) || other.cvss == cvss) &&
            (identical(other.summary, summary) || other.summary == summary) &&
            const DeepCollectionEquality().equals(other._actions, _actions));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      timestamp,
      severity,
      const DeepCollectionEquality().hash(_chain),
      const DeepCollectionEquality().hash(_entities),
      sigmaScore,
      zScore,
      iocMatch,
      cvss,
      summary,
      const DeepCollectionEquality().hash(_actions));

  /// Create a copy of Incident
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IncidentImplCopyWith<_$IncidentImpl> get copyWith =>
      __$$IncidentImplCopyWithImpl<_$IncidentImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$IncidentImplToJson(
      this,
    );
  }
}

abstract class _Incident implements Incident {
  const factory _Incident(
      {required final String id,
      required final DateTime timestamp,
      required final String severity,
      final List<String> chain,
      final List<String> entities,
      final double sigmaScore,
      final double zScore,
      final String? iocMatch,
      final double cvss,
      final String? summary,
      final List<String>? actions}) = _$IncidentImpl;

  factory _Incident.fromJson(Map<String, dynamic> json) =
      _$IncidentImpl.fromJson;

  @override
  String get id;
  @override
  DateTime get timestamp;
  @override
  String get severity;
  @override
  List<String> get chain;
  @override
  List<String> get entities;
  @override
  double get sigmaScore;
  @override
  double get zScore;
  @override
  String? get iocMatch;
  @override
  double get cvss;
  @override
  String? get summary;
  @override
  List<String>? get actions;

  /// Create a copy of Incident
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IncidentImplCopyWith<_$IncidentImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
