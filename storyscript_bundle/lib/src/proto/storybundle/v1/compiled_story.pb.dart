// This is a generated file - do not edit.
//
// Generated from storybundle/v1/compiled_story.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart' as $0;

import 'compiled_story.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'compiled_story.pbenum.dart';

/// Canonical, source-free StoryScript semantic model for .storybundle v1.
class CompiledStory extends $pb.GeneratedMessage {
  factory CompiledStory({
    $core.int? formatVersion,
    ProjectMetadata? project,
    Initialization? initialization,
    $core.Iterable<LogicBlock>? logicBlocks,
    $core.Iterable<Scene>? scenes,
  }) {
    final result = CompiledStory._();
    if (formatVersion != null) result.formatVersion = formatVersion;
    if (project != null) result.project = project;
    if (initialization != null) result.initialization = initialization;
    if (logicBlocks != null) result.logicBlocks.addAll(logicBlocks);
    if (scenes != null) result.scenes.addAll(scenes);
    return result;
  }

  CompiledStory._();

  factory CompiledStory.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompiledStory()..mergeFromBuffer(data, registry);
  factory CompiledStory.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompiledStory()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompiledStory',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: CompiledStory.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'formatVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<ProjectMetadata>(2, _omitFieldNames ? '' : 'project',
        subBuilder: ProjectMetadata.$_createMessage)
    ..aOM<Initialization>(3, _omitFieldNames ? '' : 'initialization',
        subBuilder: Initialization.$_createMessage)
    ..pPM<LogicBlock>(4, _omitFieldNames ? '' : 'logicBlocks',
        subBuilder: LogicBlock.$_createMessage)
    ..pPM<Scene>(5, _omitFieldNames ? '' : 'scenes',
        subBuilder: Scene.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompiledStory clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompiledStory copyWith(void Function(CompiledStory) updates) =>
      super.copyWith((message) => updates(message as CompiledStory))
          as CompiledStory;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CompiledStory() / CompiledStory.new instead')
  static CompiledStory create() => CompiledStory._();
  static $pb.GeneratedMessage $_createMessage() => CompiledStory._();
  @$core.override
  CompiledStory createEmptyInstance() => CompiledStory._();
  @$core.pragma('dart2js:noInline')
  static CompiledStory getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CompiledStory>(
          CompiledStory.$_createMessage);
  static CompiledStory? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get formatVersion => $_getIZ(0);
  @$pb.TagNumber(1)
  set formatVersion($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFormatVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearFormatVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  ProjectMetadata get project => $_getN(1);
  @$pb.TagNumber(2)
  set project(ProjectMetadata value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasProject() => $_has(1);
  @$pb.TagNumber(2)
  void clearProject() => $_clearField(2);
  @$pb.TagNumber(2)
  ProjectMetadata ensureProject() => $_ensure(1);

  @$pb.TagNumber(3)
  Initialization get initialization => $_getN(2);
  @$pb.TagNumber(3)
  set initialization(Initialization value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasInitialization() => $_has(2);
  @$pb.TagNumber(3)
  void clearInitialization() => $_clearField(3);
  @$pb.TagNumber(3)
  Initialization ensureInitialization() => $_ensure(2);

  @$pb.TagNumber(4)
  $pb.PbList<LogicBlock> get logicBlocks => $_getList(3);

  @$pb.TagNumber(5)
  $pb.PbList<Scene> get scenes => $_getList(4);
}

class ProjectMetadata extends $pb.GeneratedMessage {
  factory ProjectMetadata({
    $core.String? id,
    $core.String? name,
    $core.String? version,
  }) {
    final result = ProjectMetadata._();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (version != null) result.version = version;
    return result;
  }

  ProjectMetadata._();

  factory ProjectMetadata.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectMetadata()..mergeFromBuffer(data, registry);
  factory ProjectMetadata.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ProjectMetadata()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ProjectMetadata',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ProjectMetadata.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'version')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectMetadata clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ProjectMetadata copyWith(void Function(ProjectMetadata) updates) =>
      super.copyWith((message) => updates(message as ProjectMetadata))
          as ProjectMetadata;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ProjectMetadata() / ProjectMetadata.new instead')
  static ProjectMetadata create() => ProjectMetadata._();
  static $pb.GeneratedMessage $_createMessage() => ProjectMetadata._();
  @$core.override
  ProjectMetadata createEmptyInstance() => ProjectMetadata._();
  @$core.pragma('dart2js:noInline')
  static ProjectMetadata getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ProjectMetadata>(
          ProjectMetadata.$_createMessage);
  static ProjectMetadata? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get version => $_getSZ(2);
  @$pb.TagNumber(3)
  set version($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearVersion() => $_clearField(3);
}

class Initialization extends $pb.GeneratedMessage {
  factory Initialization({
    $core.Iterable<VariableDefinition>? variables,
    $core.Iterable<Actor>? actors,
    $core.String? startScene,
  }) {
    final result = Initialization._();
    if (variables != null) result.variables.addAll(variables);
    if (actors != null) result.actors.addAll(actors);
    if (startScene != null) result.startScene = startScene;
    return result;
  }

  Initialization._();

  factory Initialization.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Initialization()..mergeFromBuffer(data, registry);
  factory Initialization.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Initialization()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Initialization',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Initialization.$_createMessage)
    ..pPM<VariableDefinition>(1, _omitFieldNames ? '' : 'variables',
        subBuilder: VariableDefinition.$_createMessage)
    ..pPM<Actor>(2, _omitFieldNames ? '' : 'actors',
        subBuilder: Actor.$_createMessage)
    ..aOS(3, _omitFieldNames ? '' : 'startScene')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Initialization clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Initialization copyWith(void Function(Initialization) updates) =>
      super.copyWith((message) => updates(message as Initialization))
          as Initialization;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Initialization() / Initialization.new instead')
  static Initialization create() => Initialization._();
  static $pb.GeneratedMessage $_createMessage() => Initialization._();
  @$core.override
  Initialization createEmptyInstance() => Initialization._();
  @$core.pragma('dart2js:noInline')
  static Initialization getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Initialization>(
          Initialization.$_createMessage);
  static Initialization? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<VariableDefinition> get variables => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<Actor> get actors => $_getList(1);

  @$pb.TagNumber(3)
  $core.String get startScene => $_getSZ(2);
  @$pb.TagNumber(3)
  set startScene($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStartScene() => $_has(2);
  @$pb.TagNumber(3)
  void clearStartScene() => $_clearField(3);
}

class VariableDefinition extends $pb.GeneratedMessage {
  factory VariableDefinition({
    $core.String? name,
    VariableType? type,
    Expression? value,
  }) {
    final result = VariableDefinition._();
    if (name != null) result.name = name;
    if (type != null) result.type = type;
    if (value != null) result.value = value;
    return result;
  }

  VariableDefinition._();

  factory VariableDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableDefinition()..mergeFromBuffer(data, registry);
  factory VariableDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableDefinition()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VariableDefinition',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: VariableDefinition.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aE<VariableType>(2, _omitFieldNames ? '' : 'type',
        enumValues: VariableType.values)
    ..aOM<Expression>(3, _omitFieldNames ? '' : 'value',
        subBuilder: Expression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableDefinition copyWith(void Function(VariableDefinition) updates) =>
      super.copyWith((message) => updates(message as VariableDefinition))
          as VariableDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use VariableDefinition() / VariableDefinition.new instead')
  static VariableDefinition create() => VariableDefinition._();
  static $pb.GeneratedMessage $_createMessage() => VariableDefinition._();
  @$core.override
  VariableDefinition createEmptyInstance() => VariableDefinition._();
  @$core.pragma('dart2js:noInline')
  static VariableDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VariableDefinition>(
          VariableDefinition.$_createMessage);
  static VariableDefinition? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  VariableType get type => $_getN(1);
  @$pb.TagNumber(2)
  set type(VariableType value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasType() => $_has(1);
  @$pb.TagNumber(2)
  void clearType() => $_clearField(2);

  @$pb.TagNumber(3)
  Expression get value => $_getN(2);
  @$pb.TagNumber(3)
  set value(Expression value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);
  @$pb.TagNumber(3)
  Expression ensureValue() => $_ensure(2);
}

class Actor extends $pb.GeneratedMessage {
  factory Actor({
    $core.String? id,
    InterpolatedString? displayName,
    $core.Iterable<Portrait>? portraits,
  }) {
    final result = Actor._();
    if (id != null) result.id = id;
    if (displayName != null) result.displayName = displayName;
    if (portraits != null) result.portraits.addAll(portraits);
    return result;
  }

  Actor._();

  factory Actor.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Actor()..mergeFromBuffer(data, registry);
  factory Actor.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Actor()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Actor',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Actor.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOM<InterpolatedString>(2, _omitFieldNames ? '' : 'displayName',
        subBuilder: InterpolatedString.$_createMessage)
    ..pPM<Portrait>(3, _omitFieldNames ? '' : 'portraits',
        subBuilder: Portrait.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Actor clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Actor copyWith(void Function(Actor) updates) =>
      super.copyWith((message) => updates(message as Actor)) as Actor;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Actor() / Actor.new instead')
  static Actor create() => Actor._();
  static $pb.GeneratedMessage $_createMessage() => Actor._();
  @$core.override
  Actor createEmptyInstance() => Actor._();
  @$core.pragma('dart2js:noInline')
  static Actor getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Actor>(Actor.$_createMessage);
  static Actor? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  InterpolatedString get displayName => $_getN(1);
  @$pb.TagNumber(2)
  set displayName(InterpolatedString value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayName() => $_clearField(2);
  @$pb.TagNumber(2)
  InterpolatedString ensureDisplayName() => $_ensure(1);

  @$pb.TagNumber(3)
  $pb.PbList<Portrait> get portraits => $_getList(2);
}

class Portrait extends $pb.GeneratedMessage {
  factory Portrait({
    $core.String? emotion,
    InterpolatedString? assetPath,
  }) {
    final result = Portrait._();
    if (emotion != null) result.emotion = emotion;
    if (assetPath != null) result.assetPath = assetPath;
    return result;
  }

  Portrait._();

  factory Portrait.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Portrait()..mergeFromBuffer(data, registry);
  factory Portrait.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Portrait()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Portrait',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Portrait.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'emotion')
    ..aOM<InterpolatedString>(2, _omitFieldNames ? '' : 'assetPath',
        subBuilder: InterpolatedString.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Portrait clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Portrait copyWith(void Function(Portrait) updates) =>
      super.copyWith((message) => updates(message as Portrait)) as Portrait;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Portrait() / Portrait.new instead')
  static Portrait create() => Portrait._();
  static $pb.GeneratedMessage $_createMessage() => Portrait._();
  @$core.override
  Portrait createEmptyInstance() => Portrait._();
  @$core.pragma('dart2js:noInline')
  static Portrait getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Portrait>(Portrait.$_createMessage);
  static Portrait? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get emotion => $_getSZ(0);
  @$pb.TagNumber(1)
  set emotion($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEmotion() => $_has(0);
  @$pb.TagNumber(1)
  void clearEmotion() => $_clearField(1);

  @$pb.TagNumber(2)
  InterpolatedString get assetPath => $_getN(1);
  @$pb.TagNumber(2)
  set assetPath(InterpolatedString value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasAssetPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearAssetPath() => $_clearField(2);
  @$pb.TagNumber(2)
  InterpolatedString ensureAssetPath() => $_ensure(1);
}

class InterpolatedString extends $pb.GeneratedMessage {
  factory InterpolatedString({
    $core.Iterable<StringSegment>? segments,
  }) {
    final result = InterpolatedString._();
    if (segments != null) result.segments.addAll(segments);
    return result;
  }

  InterpolatedString._();

  factory InterpolatedString.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InterpolatedString()..mergeFromBuffer(data, registry);
  factory InterpolatedString.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InterpolatedString()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InterpolatedString',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: InterpolatedString.$_createMessage)
    ..pPM<StringSegment>(1, _omitFieldNames ? '' : 'segments',
        subBuilder: StringSegment.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InterpolatedString clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InterpolatedString copyWith(void Function(InterpolatedString) updates) =>
      super.copyWith((message) => updates(message as InterpolatedString))
          as InterpolatedString;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use InterpolatedString() / InterpolatedString.new instead')
  static InterpolatedString create() => InterpolatedString._();
  static $pb.GeneratedMessage $_createMessage() => InterpolatedString._();
  @$core.override
  InterpolatedString createEmptyInstance() => InterpolatedString._();
  @$core.pragma('dart2js:noInline')
  static InterpolatedString getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InterpolatedString>(
          InterpolatedString.$_createMessage);
  static InterpolatedString? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<StringSegment> get segments => $_getList(0);
}

enum StringSegment_Value { literal, variable, notSet }

class StringSegment extends $pb.GeneratedMessage {
  factory StringSegment({
    $core.String? literal,
    $core.String? variable,
  }) {
    final result = StringSegment._();
    if (literal != null) result.literal = literal;
    if (variable != null) result.variable = variable;
    return result;
  }

  StringSegment._();

  factory StringSegment.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StringSegment()..mergeFromBuffer(data, registry);
  factory StringSegment.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StringSegment()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, StringSegment_Value>
      _StringSegment_ValueByTag = {
    1: StringSegment_Value.literal,
    2: StringSegment_Value.variable,
    0: StringSegment_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StringSegment',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StringSegment.$_createMessage)
    ..oo(0, [1, 2])
    ..aOS(1, _omitFieldNames ? '' : 'literal')
    ..aOS(2, _omitFieldNames ? '' : 'variable')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringSegment clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StringSegment copyWith(void Function(StringSegment) updates) =>
      super.copyWith((message) => updates(message as StringSegment))
          as StringSegment;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StringSegment() / StringSegment.new instead')
  static StringSegment create() => StringSegment._();
  static $pb.GeneratedMessage $_createMessage() => StringSegment._();
  @$core.override
  StringSegment createEmptyInstance() => StringSegment._();
  @$core.pragma('dart2js:noInline')
  static StringSegment getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StringSegment>(
          StringSegment.$_createMessage);
  static StringSegment? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  StringSegment_Value whichValue() =>
      _StringSegment_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get literal => $_getSZ(0);
  @$pb.TagNumber(1)
  set literal($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLiteral() => $_has(0);
  @$pb.TagNumber(1)
  void clearLiteral() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get variable => $_getSZ(1);
  @$pb.TagNumber(2)
  set variable($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVariable() => $_has(1);
  @$pb.TagNumber(2)
  void clearVariable() => $_clearField(2);
}

class LogicBlock extends $pb.GeneratedMessage {
  factory LogicBlock({
    $core.String? name,
    $core.Iterable<LogicParameter>? parameters,
    VariableType? returnType,
    $core.Iterable<PrepStatement>? body,
  }) {
    final result = LogicBlock._();
    if (name != null) result.name = name;
    if (parameters != null) result.parameters.addAll(parameters);
    if (returnType != null) result.returnType = returnType;
    if (body != null) result.body.addAll(body);
    return result;
  }

  LogicBlock._();

  factory LogicBlock.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LogicBlock()..mergeFromBuffer(data, registry);
  factory LogicBlock.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LogicBlock()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LogicBlock',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: LogicBlock.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..pPM<LogicParameter>(2, _omitFieldNames ? '' : 'parameters',
        subBuilder: LogicParameter.$_createMessage)
    ..aE<VariableType>(3, _omitFieldNames ? '' : 'returnType',
        enumValues: VariableType.values)
    ..pPM<PrepStatement>(4, _omitFieldNames ? '' : 'body',
        subBuilder: PrepStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LogicBlock clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LogicBlock copyWith(void Function(LogicBlock) updates) =>
      super.copyWith((message) => updates(message as LogicBlock)) as LogicBlock;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use LogicBlock() / LogicBlock.new instead')
  static LogicBlock create() => LogicBlock._();
  static $pb.GeneratedMessage $_createMessage() => LogicBlock._();
  @$core.override
  LogicBlock createEmptyInstance() => LogicBlock._();
  @$core.pragma('dart2js:noInline')
  static LogicBlock getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LogicBlock>(LogicBlock.$_createMessage);
  static LogicBlock? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<LogicParameter> get parameters => $_getList(1);

  @$pb.TagNumber(3)
  VariableType get returnType => $_getN(2);
  @$pb.TagNumber(3)
  set returnType(VariableType value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasReturnType() => $_has(2);
  @$pb.TagNumber(3)
  void clearReturnType() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<PrepStatement> get body => $_getList(3);
}

class LogicParameter extends $pb.GeneratedMessage {
  factory LogicParameter({
    $core.String? name,
    VariableType? type,
  }) {
    final result = LogicParameter._();
    if (name != null) result.name = name;
    if (type != null) result.type = type;
    return result;
  }

  LogicParameter._();

  factory LogicParameter.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LogicParameter()..mergeFromBuffer(data, registry);
  factory LogicParameter.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LogicParameter()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LogicParameter',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: LogicParameter.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aE<VariableType>(2, _omitFieldNames ? '' : 'type',
        enumValues: VariableType.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LogicParameter clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LogicParameter copyWith(void Function(LogicParameter) updates) =>
      super.copyWith((message) => updates(message as LogicParameter))
          as LogicParameter;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use LogicParameter() / LogicParameter.new instead')
  static LogicParameter create() => LogicParameter._();
  static $pb.GeneratedMessage $_createMessage() => LogicParameter._();
  @$core.override
  LogicParameter createEmptyInstance() => LogicParameter._();
  @$core.pragma('dart2js:noInline')
  static LogicParameter getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<LogicParameter>(
          LogicParameter.$_createMessage);
  static LogicParameter? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  VariableType get type => $_getN(1);
  @$pb.TagNumber(2)
  set type(VariableType value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasType() => $_has(1);
  @$pb.TagNumber(2)
  void clearType() => $_clearField(2);
}

class Scene extends $pb.GeneratedMessage {
  factory Scene({
    $core.String? label,
    PrepBlock? prep,
    StoryBlock? story,
  }) {
    final result = Scene._();
    if (label != null) result.label = label;
    if (prep != null) result.prep = prep;
    if (story != null) result.story = story;
    return result;
  }

  Scene._();

  factory Scene.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Scene()..mergeFromBuffer(data, registry);
  factory Scene.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Scene()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Scene',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Scene.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'label')
    ..aOM<PrepBlock>(2, _omitFieldNames ? '' : 'prep',
        subBuilder: PrepBlock.$_createMessage)
    ..aOM<StoryBlock>(3, _omitFieldNames ? '' : 'story',
        subBuilder: StoryBlock.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Scene clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Scene copyWith(void Function(Scene) updates) =>
      super.copyWith((message) => updates(message as Scene)) as Scene;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Scene() / Scene.new instead')
  static Scene create() => Scene._();
  static $pb.GeneratedMessage $_createMessage() => Scene._();
  @$core.override
  Scene createEmptyInstance() => Scene._();
  @$core.pragma('dart2js:noInline')
  static Scene getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Scene>(Scene.$_createMessage);
  static Scene? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get label => $_getSZ(0);
  @$pb.TagNumber(1)
  set label($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLabel() => $_has(0);
  @$pb.TagNumber(1)
  void clearLabel() => $_clearField(1);

  @$pb.TagNumber(2)
  PrepBlock get prep => $_getN(1);
  @$pb.TagNumber(2)
  set prep(PrepBlock value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPrep() => $_has(1);
  @$pb.TagNumber(2)
  void clearPrep() => $_clearField(2);
  @$pb.TagNumber(2)
  PrepBlock ensurePrep() => $_ensure(1);

  @$pb.TagNumber(3)
  StoryBlock get story => $_getN(2);
  @$pb.TagNumber(3)
  set story(StoryBlock value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasStory() => $_has(2);
  @$pb.TagNumber(3)
  void clearStory() => $_clearField(3);
  @$pb.TagNumber(3)
  StoryBlock ensureStory() => $_ensure(2);
}

class PrepBlock extends $pb.GeneratedMessage {
  factory PrepBlock({
    $core.Iterable<PrepStatement>? statements,
  }) {
    final result = PrepBlock._();
    if (statements != null) result.statements.addAll(statements);
    return result;
  }

  PrepBlock._();

  factory PrepBlock.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepBlock()..mergeFromBuffer(data, registry);
  factory PrepBlock.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepBlock()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PrepBlock',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PrepBlock.$_createMessage)
    ..pPM<PrepStatement>(1, _omitFieldNames ? '' : 'statements',
        subBuilder: PrepStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepBlock clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepBlock copyWith(void Function(PrepBlock) updates) =>
      super.copyWith((message) => updates(message as PrepBlock)) as PrepBlock;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PrepBlock() / PrepBlock.new instead')
  static PrepBlock create() => PrepBlock._();
  static $pb.GeneratedMessage $_createMessage() => PrepBlock._();
  @$core.override
  PrepBlock createEmptyInstance() => PrepBlock._();
  @$core.pragma('dart2js:noInline')
  static PrepBlock getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PrepBlock>(PrepBlock.$_createMessage);
  static PrepBlock? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<PrepStatement> get statements => $_getList(0);
}

class StoryBlock extends $pb.GeneratedMessage {
  factory StoryBlock({
    $core.Iterable<StoryStatement>? statements,
  }) {
    final result = StoryBlock._();
    if (statements != null) result.statements.addAll(statements);
    return result;
  }

  StoryBlock._();

  factory StoryBlock.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryBlock()..mergeFromBuffer(data, registry);
  factory StoryBlock.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryBlock()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoryBlock',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StoryBlock.$_createMessage)
    ..pPM<StoryStatement>(1, _omitFieldNames ? '' : 'statements',
        subBuilder: StoryStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryBlock clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryBlock copyWith(void Function(StoryBlock) updates) =>
      super.copyWith((message) => updates(message as StoryBlock)) as StoryBlock;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StoryBlock() / StoryBlock.new instead')
  static StoryBlock create() => StoryBlock._();
  static $pb.GeneratedMessage $_createMessage() => StoryBlock._();
  @$core.override
  StoryBlock createEmptyInstance() => StoryBlock._();
  @$core.pragma('dart2js:noInline')
  static StoryBlock getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StoryBlock>(StoryBlock.$_createMessage);
  static StoryBlock? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<StoryStatement> get statements => $_getList(0);
}

enum PrepStatement_Value {
  background,
  bgm,
  sfx,
  variableDefinition,
  variableAssignment,
  call,
  ifElse,
  forSnapshot,
  repeat,
  breakLoop,
  continueLoop,
  returnStatement,
  notSet
}

class PrepStatement extends $pb.GeneratedMessage {
  factory PrepStatement({
    BackgroundDirective? background,
    BgmDirective? bgm,
    SfxDirective? sfx,
    VariableDefinition? variableDefinition,
    VariableAssignment? variableAssignment,
    CallExpression? call,
    PrepIfElse? ifElse,
    PrepForSnapshot? forSnapshot,
    PrepRepeat? repeat,
    $0.Empty? breakLoop,
    $0.Empty? continueLoop,
    ReturnStatement? returnStatement,
  }) {
    final result = PrepStatement._();
    if (background != null) result.background = background;
    if (bgm != null) result.bgm = bgm;
    if (sfx != null) result.sfx = sfx;
    if (variableDefinition != null)
      result.variableDefinition = variableDefinition;
    if (variableAssignment != null)
      result.variableAssignment = variableAssignment;
    if (call != null) result.call = call;
    if (ifElse != null) result.ifElse = ifElse;
    if (forSnapshot != null) result.forSnapshot = forSnapshot;
    if (repeat != null) result.repeat = repeat;
    if (breakLoop != null) result.breakLoop = breakLoop;
    if (continueLoop != null) result.continueLoop = continueLoop;
    if (returnStatement != null) result.returnStatement = returnStatement;
    return result;
  }

  PrepStatement._();

  factory PrepStatement.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepStatement()..mergeFromBuffer(data, registry);
  factory PrepStatement.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepStatement()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, PrepStatement_Value>
      _PrepStatement_ValueByTag = {
    1: PrepStatement_Value.background,
    2: PrepStatement_Value.bgm,
    3: PrepStatement_Value.sfx,
    4: PrepStatement_Value.variableDefinition,
    5: PrepStatement_Value.variableAssignment,
    6: PrepStatement_Value.call,
    7: PrepStatement_Value.ifElse,
    8: PrepStatement_Value.forSnapshot,
    9: PrepStatement_Value.repeat,
    10: PrepStatement_Value.breakLoop,
    11: PrepStatement_Value.continueLoop,
    12: PrepStatement_Value.returnStatement,
    0: PrepStatement_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PrepStatement',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PrepStatement.$_createMessage)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
    ..aOM<BackgroundDirective>(1, _omitFieldNames ? '' : 'background',
        subBuilder: BackgroundDirective.$_createMessage)
    ..aOM<BgmDirective>(2, _omitFieldNames ? '' : 'bgm',
        subBuilder: BgmDirective.$_createMessage)
    ..aOM<SfxDirective>(3, _omitFieldNames ? '' : 'sfx',
        subBuilder: SfxDirective.$_createMessage)
    ..aOM<VariableDefinition>(4, _omitFieldNames ? '' : 'variableDefinition',
        subBuilder: VariableDefinition.$_createMessage)
    ..aOM<VariableAssignment>(5, _omitFieldNames ? '' : 'variableAssignment',
        subBuilder: VariableAssignment.$_createMessage)
    ..aOM<CallExpression>(6, _omitFieldNames ? '' : 'call',
        subBuilder: CallExpression.$_createMessage)
    ..aOM<PrepIfElse>(7, _omitFieldNames ? '' : 'ifElse',
        subBuilder: PrepIfElse.$_createMessage)
    ..aOM<PrepForSnapshot>(8, _omitFieldNames ? '' : 'forSnapshot',
        subBuilder: PrepForSnapshot.$_createMessage)
    ..aOM<PrepRepeat>(9, _omitFieldNames ? '' : 'repeat',
        subBuilder: PrepRepeat.$_createMessage)
    ..aOM<$0.Empty>(10, _omitFieldNames ? '' : 'breakLoop',
        subBuilder: $0.Empty.$_createMessage)
    ..aOM<$0.Empty>(11, _omitFieldNames ? '' : 'continueLoop',
        subBuilder: $0.Empty.$_createMessage)
    ..aOM<ReturnStatement>(12, _omitFieldNames ? '' : 'returnStatement',
        subBuilder: ReturnStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepStatement clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepStatement copyWith(void Function(PrepStatement) updates) =>
      super.copyWith((message) => updates(message as PrepStatement))
          as PrepStatement;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PrepStatement() / PrepStatement.new instead')
  static PrepStatement create() => PrepStatement._();
  static $pb.GeneratedMessage $_createMessage() => PrepStatement._();
  @$core.override
  PrepStatement createEmptyInstance() => PrepStatement._();
  @$core.pragma('dart2js:noInline')
  static PrepStatement getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PrepStatement>(
          PrepStatement.$_createMessage);
  static PrepStatement? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  PrepStatement_Value whichValue() =>
      _PrepStatement_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  BackgroundDirective get background => $_getN(0);
  @$pb.TagNumber(1)
  set background(BackgroundDirective value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasBackground() => $_has(0);
  @$pb.TagNumber(1)
  void clearBackground() => $_clearField(1);
  @$pb.TagNumber(1)
  BackgroundDirective ensureBackground() => $_ensure(0);

  @$pb.TagNumber(2)
  BgmDirective get bgm => $_getN(1);
  @$pb.TagNumber(2)
  set bgm(BgmDirective value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasBgm() => $_has(1);
  @$pb.TagNumber(2)
  void clearBgm() => $_clearField(2);
  @$pb.TagNumber(2)
  BgmDirective ensureBgm() => $_ensure(1);

  @$pb.TagNumber(3)
  SfxDirective get sfx => $_getN(2);
  @$pb.TagNumber(3)
  set sfx(SfxDirective value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasSfx() => $_has(2);
  @$pb.TagNumber(3)
  void clearSfx() => $_clearField(3);
  @$pb.TagNumber(3)
  SfxDirective ensureSfx() => $_ensure(2);

  @$pb.TagNumber(4)
  VariableDefinition get variableDefinition => $_getN(3);
  @$pb.TagNumber(4)
  set variableDefinition(VariableDefinition value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasVariableDefinition() => $_has(3);
  @$pb.TagNumber(4)
  void clearVariableDefinition() => $_clearField(4);
  @$pb.TagNumber(4)
  VariableDefinition ensureVariableDefinition() => $_ensure(3);

  @$pb.TagNumber(5)
  VariableAssignment get variableAssignment => $_getN(4);
  @$pb.TagNumber(5)
  set variableAssignment(VariableAssignment value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasVariableAssignment() => $_has(4);
  @$pb.TagNumber(5)
  void clearVariableAssignment() => $_clearField(5);
  @$pb.TagNumber(5)
  VariableAssignment ensureVariableAssignment() => $_ensure(4);

  @$pb.TagNumber(6)
  CallExpression get call => $_getN(5);
  @$pb.TagNumber(6)
  set call(CallExpression value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasCall() => $_has(5);
  @$pb.TagNumber(6)
  void clearCall() => $_clearField(6);
  @$pb.TagNumber(6)
  CallExpression ensureCall() => $_ensure(5);

  @$pb.TagNumber(7)
  PrepIfElse get ifElse => $_getN(6);
  @$pb.TagNumber(7)
  set ifElse(PrepIfElse value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasIfElse() => $_has(6);
  @$pb.TagNumber(7)
  void clearIfElse() => $_clearField(7);
  @$pb.TagNumber(7)
  PrepIfElse ensureIfElse() => $_ensure(6);

  @$pb.TagNumber(8)
  PrepForSnapshot get forSnapshot => $_getN(7);
  @$pb.TagNumber(8)
  set forSnapshot(PrepForSnapshot value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasForSnapshot() => $_has(7);
  @$pb.TagNumber(8)
  void clearForSnapshot() => $_clearField(8);
  @$pb.TagNumber(8)
  PrepForSnapshot ensureForSnapshot() => $_ensure(7);

  @$pb.TagNumber(9)
  PrepRepeat get repeat => $_getN(8);
  @$pb.TagNumber(9)
  set repeat(PrepRepeat value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasRepeat() => $_has(8);
  @$pb.TagNumber(9)
  void clearRepeat() => $_clearField(9);
  @$pb.TagNumber(9)
  PrepRepeat ensureRepeat() => $_ensure(8);

  @$pb.TagNumber(10)
  $0.Empty get breakLoop => $_getN(9);
  @$pb.TagNumber(10)
  set breakLoop($0.Empty value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasBreakLoop() => $_has(9);
  @$pb.TagNumber(10)
  void clearBreakLoop() => $_clearField(10);
  @$pb.TagNumber(10)
  $0.Empty ensureBreakLoop() => $_ensure(9);

  @$pb.TagNumber(11)
  $0.Empty get continueLoop => $_getN(10);
  @$pb.TagNumber(11)
  set continueLoop($0.Empty value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasContinueLoop() => $_has(10);
  @$pb.TagNumber(11)
  void clearContinueLoop() => $_clearField(11);
  @$pb.TagNumber(11)
  $0.Empty ensureContinueLoop() => $_ensure(10);

  @$pb.TagNumber(12)
  ReturnStatement get returnStatement => $_getN(11);
  @$pb.TagNumber(12)
  set returnStatement(ReturnStatement value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasReturnStatement() => $_has(11);
  @$pb.TagNumber(12)
  void clearReturnStatement() => $_clearField(12);
  @$pb.TagNumber(12)
  ReturnStatement ensureReturnStatement() => $_ensure(11);
}

class BackgroundDirective extends $pb.GeneratedMessage {
  factory BackgroundDirective({
    InterpolatedString? assetPath,
  }) {
    final result = BackgroundDirective._();
    if (assetPath != null) result.assetPath = assetPath;
    return result;
  }

  BackgroundDirective._();

  factory BackgroundDirective.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BackgroundDirective()..mergeFromBuffer(data, registry);
  factory BackgroundDirective.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BackgroundDirective()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BackgroundDirective',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: BackgroundDirective.$_createMessage)
    ..aOM<InterpolatedString>(1, _omitFieldNames ? '' : 'assetPath',
        subBuilder: InterpolatedString.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BackgroundDirective clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BackgroundDirective copyWith(void Function(BackgroundDirective) updates) =>
      super.copyWith((message) => updates(message as BackgroundDirective))
          as BackgroundDirective;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use BackgroundDirective() / BackgroundDirective.new instead')
  static BackgroundDirective create() => BackgroundDirective._();
  static $pb.GeneratedMessage $_createMessage() => BackgroundDirective._();
  @$core.override
  BackgroundDirective createEmptyInstance() => BackgroundDirective._();
  @$core.pragma('dart2js:noInline')
  static BackgroundDirective getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BackgroundDirective>(
          BackgroundDirective.$_createMessage);
  static BackgroundDirective? _defaultInstance;

  @$pb.TagNumber(1)
  InterpolatedString get assetPath => $_getN(0);
  @$pb.TagNumber(1)
  set assetPath(InterpolatedString value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAssetPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearAssetPath() => $_clearField(1);
  @$pb.TagNumber(1)
  InterpolatedString ensureAssetPath() => $_ensure(0);
}

enum BgmDirective_Value { assetPath, stop, notSet }

class BgmDirective extends $pb.GeneratedMessage {
  factory BgmDirective({
    InterpolatedString? assetPath,
    $0.Empty? stop,
  }) {
    final result = BgmDirective._();
    if (assetPath != null) result.assetPath = assetPath;
    if (stop != null) result.stop = stop;
    return result;
  }

  BgmDirective._();

  factory BgmDirective.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BgmDirective()..mergeFromBuffer(data, registry);
  factory BgmDirective.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BgmDirective()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, BgmDirective_Value>
      _BgmDirective_ValueByTag = {
    1: BgmDirective_Value.assetPath,
    2: BgmDirective_Value.stop,
    0: BgmDirective_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BgmDirective',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: BgmDirective.$_createMessage)
    ..oo(0, [1, 2])
    ..aOM<InterpolatedString>(1, _omitFieldNames ? '' : 'assetPath',
        subBuilder: InterpolatedString.$_createMessage)
    ..aOM<$0.Empty>(2, _omitFieldNames ? '' : 'stop',
        subBuilder: $0.Empty.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BgmDirective clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BgmDirective copyWith(void Function(BgmDirective) updates) =>
      super.copyWith((message) => updates(message as BgmDirective))
          as BgmDirective;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use BgmDirective() / BgmDirective.new instead')
  static BgmDirective create() => BgmDirective._();
  static $pb.GeneratedMessage $_createMessage() => BgmDirective._();
  @$core.override
  BgmDirective createEmptyInstance() => BgmDirective._();
  @$core.pragma('dart2js:noInline')
  static BgmDirective getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<BgmDirective>(
          BgmDirective.$_createMessage);
  static BgmDirective? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  BgmDirective_Value whichValue() => _BgmDirective_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  InterpolatedString get assetPath => $_getN(0);
  @$pb.TagNumber(1)
  set assetPath(InterpolatedString value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAssetPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearAssetPath() => $_clearField(1);
  @$pb.TagNumber(1)
  InterpolatedString ensureAssetPath() => $_ensure(0);

  @$pb.TagNumber(2)
  $0.Empty get stop => $_getN(1);
  @$pb.TagNumber(2)
  set stop($0.Empty value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasStop() => $_has(1);
  @$pb.TagNumber(2)
  void clearStop() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.Empty ensureStop() => $_ensure(1);
}

class SfxDirective extends $pb.GeneratedMessage {
  factory SfxDirective({
    InterpolatedString? assetPath,
  }) {
    final result = SfxDirective._();
    if (assetPath != null) result.assetPath = assetPath;
    return result;
  }

  SfxDirective._();

  factory SfxDirective.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SfxDirective()..mergeFromBuffer(data, registry);
  factory SfxDirective.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SfxDirective()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SfxDirective',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: SfxDirective.$_createMessage)
    ..aOM<InterpolatedString>(1, _omitFieldNames ? '' : 'assetPath',
        subBuilder: InterpolatedString.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SfxDirective clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SfxDirective copyWith(void Function(SfxDirective) updates) =>
      super.copyWith((message) => updates(message as SfxDirective))
          as SfxDirective;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SfxDirective() / SfxDirective.new instead')
  static SfxDirective create() => SfxDirective._();
  static $pb.GeneratedMessage $_createMessage() => SfxDirective._();
  @$core.override
  SfxDirective createEmptyInstance() => SfxDirective._();
  @$core.pragma('dart2js:noInline')
  static SfxDirective getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SfxDirective>(
          SfxDirective.$_createMessage);
  static SfxDirective? _defaultInstance;

  @$pb.TagNumber(1)
  InterpolatedString get assetPath => $_getN(0);
  @$pb.TagNumber(1)
  set assetPath(InterpolatedString value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAssetPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearAssetPath() => $_clearField(1);
  @$pb.TagNumber(1)
  InterpolatedString ensureAssetPath() => $_ensure(0);
}

class VariableAssignment extends $pb.GeneratedMessage {
  factory VariableAssignment({
    $core.String? name,
    AssignmentOperator? operator,
    Expression? value,
  }) {
    final result = VariableAssignment._();
    if (name != null) result.name = name;
    if (operator != null) result.operator = operator;
    if (value != null) result.value = value;
    return result;
  }

  VariableAssignment._();

  factory VariableAssignment.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableAssignment()..mergeFromBuffer(data, registry);
  factory VariableAssignment.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableAssignment()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VariableAssignment',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: VariableAssignment.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aE<AssignmentOperator>(2, _omitFieldNames ? '' : 'operator',
        enumValues: AssignmentOperator.values)
    ..aOM<Expression>(3, _omitFieldNames ? '' : 'value',
        subBuilder: Expression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableAssignment clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableAssignment copyWith(void Function(VariableAssignment) updates) =>
      super.copyWith((message) => updates(message as VariableAssignment))
          as VariableAssignment;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use VariableAssignment() / VariableAssignment.new instead')
  static VariableAssignment create() => VariableAssignment._();
  static $pb.GeneratedMessage $_createMessage() => VariableAssignment._();
  @$core.override
  VariableAssignment createEmptyInstance() => VariableAssignment._();
  @$core.pragma('dart2js:noInline')
  static VariableAssignment getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VariableAssignment>(
          VariableAssignment.$_createMessage);
  static VariableAssignment? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  AssignmentOperator get operator => $_getN(1);
  @$pb.TagNumber(2)
  set operator(AssignmentOperator value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasOperator() => $_has(1);
  @$pb.TagNumber(2)
  void clearOperator() => $_clearField(2);

  @$pb.TagNumber(3)
  Expression get value => $_getN(2);
  @$pb.TagNumber(3)
  set value(Expression value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);
  @$pb.TagNumber(3)
  Expression ensureValue() => $_ensure(2);
}

class PrepStatementList extends $pb.GeneratedMessage {
  factory PrepStatementList({
    $core.Iterable<PrepStatement>? statements,
  }) {
    final result = PrepStatementList._();
    if (statements != null) result.statements.addAll(statements);
    return result;
  }

  PrepStatementList._();

  factory PrepStatementList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepStatementList()..mergeFromBuffer(data, registry);
  factory PrepStatementList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepStatementList()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PrepStatementList',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PrepStatementList.$_createMessage)
    ..pPM<PrepStatement>(1, _omitFieldNames ? '' : 'statements',
        subBuilder: PrepStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepStatementList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepStatementList copyWith(void Function(PrepStatementList) updates) =>
      super.copyWith((message) => updates(message as PrepStatementList))
          as PrepStatementList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PrepStatementList() / PrepStatementList.new instead')
  static PrepStatementList create() => PrepStatementList._();
  static $pb.GeneratedMessage $_createMessage() => PrepStatementList._();
  @$core.override
  PrepStatementList createEmptyInstance() => PrepStatementList._();
  @$core.pragma('dart2js:noInline')
  static PrepStatementList getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PrepStatementList>(
          PrepStatementList.$_createMessage);
  static PrepStatementList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<PrepStatement> get statements => $_getList(0);
}

class PrepIfElse extends $pb.GeneratedMessage {
  factory PrepIfElse({
    Expression? condition,
    PrepStatementList? thenBranch,
    PrepStatementList? elseBranch,
  }) {
    final result = PrepIfElse._();
    if (condition != null) result.condition = condition;
    if (thenBranch != null) result.thenBranch = thenBranch;
    if (elseBranch != null) result.elseBranch = elseBranch;
    return result;
  }

  PrepIfElse._();

  factory PrepIfElse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepIfElse()..mergeFromBuffer(data, registry);
  factory PrepIfElse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepIfElse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PrepIfElse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PrepIfElse.$_createMessage)
    ..aOM<Expression>(1, _omitFieldNames ? '' : 'condition',
        subBuilder: Expression.$_createMessage)
    ..aOM<PrepStatementList>(2, _omitFieldNames ? '' : 'thenBranch',
        subBuilder: PrepStatementList.$_createMessage)
    ..aOM<PrepStatementList>(3, _omitFieldNames ? '' : 'elseBranch',
        subBuilder: PrepStatementList.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepIfElse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepIfElse copyWith(void Function(PrepIfElse) updates) =>
      super.copyWith((message) => updates(message as PrepIfElse)) as PrepIfElse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PrepIfElse() / PrepIfElse.new instead')
  static PrepIfElse create() => PrepIfElse._();
  static $pb.GeneratedMessage $_createMessage() => PrepIfElse._();
  @$core.override
  PrepIfElse createEmptyInstance() => PrepIfElse._();
  @$core.pragma('dart2js:noInline')
  static PrepIfElse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PrepIfElse>(PrepIfElse.$_createMessage);
  static PrepIfElse? _defaultInstance;

  @$pb.TagNumber(1)
  Expression get condition => $_getN(0);
  @$pb.TagNumber(1)
  set condition(Expression value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCondition() => $_has(0);
  @$pb.TagNumber(1)
  void clearCondition() => $_clearField(1);
  @$pb.TagNumber(1)
  Expression ensureCondition() => $_ensure(0);

  @$pb.TagNumber(2)
  PrepStatementList get thenBranch => $_getN(1);
  @$pb.TagNumber(2)
  set thenBranch(PrepStatementList value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasThenBranch() => $_has(1);
  @$pb.TagNumber(2)
  void clearThenBranch() => $_clearField(2);
  @$pb.TagNumber(2)
  PrepStatementList ensureThenBranch() => $_ensure(1);

  @$pb.TagNumber(3)
  PrepStatementList get elseBranch => $_getN(2);
  @$pb.TagNumber(3)
  set elseBranch(PrepStatementList value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasElseBranch() => $_has(2);
  @$pb.TagNumber(3)
  void clearElseBranch() => $_clearField(3);
  @$pb.TagNumber(3)
  PrepStatementList ensureElseBranch() => $_ensure(2);
}

class PrepForSnapshot extends $pb.GeneratedMessage {
  factory PrepForSnapshot({
    $core.String? itemName,
    $core.String? arrayName,
    $core.Iterable<PrepStatement>? body,
  }) {
    final result = PrepForSnapshot._();
    if (itemName != null) result.itemName = itemName;
    if (arrayName != null) result.arrayName = arrayName;
    if (body != null) result.body.addAll(body);
    return result;
  }

  PrepForSnapshot._();

  factory PrepForSnapshot.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepForSnapshot()..mergeFromBuffer(data, registry);
  factory PrepForSnapshot.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepForSnapshot()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PrepForSnapshot',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PrepForSnapshot.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'itemName')
    ..aOS(2, _omitFieldNames ? '' : 'arrayName')
    ..pPM<PrepStatement>(3, _omitFieldNames ? '' : 'body',
        subBuilder: PrepStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepForSnapshot clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepForSnapshot copyWith(void Function(PrepForSnapshot) updates) =>
      super.copyWith((message) => updates(message as PrepForSnapshot))
          as PrepForSnapshot;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PrepForSnapshot() / PrepForSnapshot.new instead')
  static PrepForSnapshot create() => PrepForSnapshot._();
  static $pb.GeneratedMessage $_createMessage() => PrepForSnapshot._();
  @$core.override
  PrepForSnapshot createEmptyInstance() => PrepForSnapshot._();
  @$core.pragma('dart2js:noInline')
  static PrepForSnapshot getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PrepForSnapshot>(
          PrepForSnapshot.$_createMessage);
  static PrepForSnapshot? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemName => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemName() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get arrayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set arrayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasArrayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearArrayName() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<PrepStatement> get body => $_getList(2);
}

class PrepRepeat extends $pb.GeneratedMessage {
  factory PrepRepeat({
    RepeatCount? count,
    $core.Iterable<PrepStatement>? body,
  }) {
    final result = PrepRepeat._();
    if (count != null) result.count = count;
    if (body != null) result.body.addAll(body);
    return result;
  }

  PrepRepeat._();

  factory PrepRepeat.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepRepeat()..mergeFromBuffer(data, registry);
  factory PrepRepeat.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PrepRepeat()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PrepRepeat',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PrepRepeat.$_createMessage)
    ..aOM<RepeatCount>(1, _omitFieldNames ? '' : 'count',
        subBuilder: RepeatCount.$_createMessage)
    ..pPM<PrepStatement>(2, _omitFieldNames ? '' : 'body',
        subBuilder: PrepStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepRepeat clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PrepRepeat copyWith(void Function(PrepRepeat) updates) =>
      super.copyWith((message) => updates(message as PrepRepeat)) as PrepRepeat;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PrepRepeat() / PrepRepeat.new instead')
  static PrepRepeat create() => PrepRepeat._();
  static $pb.GeneratedMessage $_createMessage() => PrepRepeat._();
  @$core.override
  PrepRepeat createEmptyInstance() => PrepRepeat._();
  @$core.pragma('dart2js:noInline')
  static PrepRepeat getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PrepRepeat>(PrepRepeat.$_createMessage);
  static PrepRepeat? _defaultInstance;

  @$pb.TagNumber(1)
  RepeatCount get count => $_getN(0);
  @$pb.TagNumber(1)
  set count(RepeatCount value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => $_clearField(1);
  @$pb.TagNumber(1)
  RepeatCount ensureCount() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<PrepStatement> get body => $_getList(1);
}

class ReturnStatement extends $pb.GeneratedMessage {
  factory ReturnStatement({
    Expression? value,
  }) {
    final result = ReturnStatement._();
    if (value != null) result.value = value;
    return result;
  }

  ReturnStatement._();

  factory ReturnStatement.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReturnStatement()..mergeFromBuffer(data, registry);
  factory ReturnStatement.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReturnStatement()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReturnStatement',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ReturnStatement.$_createMessage)
    ..aOM<Expression>(1, _omitFieldNames ? '' : 'value',
        subBuilder: Expression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReturnStatement clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReturnStatement copyWith(void Function(ReturnStatement) updates) =>
      super.copyWith((message) => updates(message as ReturnStatement))
          as ReturnStatement;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ReturnStatement() / ReturnStatement.new instead')
  static ReturnStatement create() => ReturnStatement._();
  static $pb.GeneratedMessage $_createMessage() => ReturnStatement._();
  @$core.override
  ReturnStatement createEmptyInstance() => ReturnStatement._();
  @$core.pragma('dart2js:noInline')
  static ReturnStatement getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ReturnStatement>(
          ReturnStatement.$_createMessage);
  static ReturnStatement? _defaultInstance;

  @$pb.TagNumber(1)
  Expression get value => $_getN(0);
  @$pb.TagNumber(1)
  set value(Expression value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearValue() => $_clearField(1);
  @$pb.TagNumber(1)
  Expression ensureValue() => $_ensure(0);
}

enum StoryStatement_Value {
  narration,
  variableOutput,
  dialogue,
  ifElse,
  choice,
  jump,
  end,
  sfx,
  forSnapshot,
  repeat,
  breakLoop,
  continueLoop,
  notSet
}

class StoryStatement extends $pb.GeneratedMessage {
  factory StoryStatement({
    Narration? narration,
    VariableOutput? variableOutput,
    Dialogue? dialogue,
    StoryIfElse? ifElse,
    ChoiceBlock? choice,
    Jump? jump,
    $0.Empty? end,
    SfxDirective? sfx,
    StoryForSnapshot? forSnapshot,
    StoryRepeat? repeat,
    $0.Empty? breakLoop,
    $0.Empty? continueLoop,
  }) {
    final result = StoryStatement._();
    if (narration != null) result.narration = narration;
    if (variableOutput != null) result.variableOutput = variableOutput;
    if (dialogue != null) result.dialogue = dialogue;
    if (ifElse != null) result.ifElse = ifElse;
    if (choice != null) result.choice = choice;
    if (jump != null) result.jump = jump;
    if (end != null) result.end = end;
    if (sfx != null) result.sfx = sfx;
    if (forSnapshot != null) result.forSnapshot = forSnapshot;
    if (repeat != null) result.repeat = repeat;
    if (breakLoop != null) result.breakLoop = breakLoop;
    if (continueLoop != null) result.continueLoop = continueLoop;
    return result;
  }

  StoryStatement._();

  factory StoryStatement.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryStatement()..mergeFromBuffer(data, registry);
  factory StoryStatement.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryStatement()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, StoryStatement_Value>
      _StoryStatement_ValueByTag = {
    1: StoryStatement_Value.narration,
    2: StoryStatement_Value.variableOutput,
    3: StoryStatement_Value.dialogue,
    4: StoryStatement_Value.ifElse,
    5: StoryStatement_Value.choice,
    6: StoryStatement_Value.jump,
    7: StoryStatement_Value.end,
    8: StoryStatement_Value.sfx,
    9: StoryStatement_Value.forSnapshot,
    10: StoryStatement_Value.repeat,
    11: StoryStatement_Value.breakLoop,
    12: StoryStatement_Value.continueLoop,
    0: StoryStatement_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoryStatement',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StoryStatement.$_createMessage)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
    ..aOM<Narration>(1, _omitFieldNames ? '' : 'narration',
        subBuilder: Narration.$_createMessage)
    ..aOM<VariableOutput>(2, _omitFieldNames ? '' : 'variableOutput',
        subBuilder: VariableOutput.$_createMessage)
    ..aOM<Dialogue>(3, _omitFieldNames ? '' : 'dialogue',
        subBuilder: Dialogue.$_createMessage)
    ..aOM<StoryIfElse>(4, _omitFieldNames ? '' : 'ifElse',
        subBuilder: StoryIfElse.$_createMessage)
    ..aOM<ChoiceBlock>(5, _omitFieldNames ? '' : 'choice',
        subBuilder: ChoiceBlock.$_createMessage)
    ..aOM<Jump>(6, _omitFieldNames ? '' : 'jump',
        subBuilder: Jump.$_createMessage)
    ..aOM<$0.Empty>(7, _omitFieldNames ? '' : 'end',
        subBuilder: $0.Empty.$_createMessage)
    ..aOM<SfxDirective>(8, _omitFieldNames ? '' : 'sfx',
        subBuilder: SfxDirective.$_createMessage)
    ..aOM<StoryForSnapshot>(9, _omitFieldNames ? '' : 'forSnapshot',
        subBuilder: StoryForSnapshot.$_createMessage)
    ..aOM<StoryRepeat>(10, _omitFieldNames ? '' : 'repeat',
        subBuilder: StoryRepeat.$_createMessage)
    ..aOM<$0.Empty>(11, _omitFieldNames ? '' : 'breakLoop',
        subBuilder: $0.Empty.$_createMessage)
    ..aOM<$0.Empty>(12, _omitFieldNames ? '' : 'continueLoop',
        subBuilder: $0.Empty.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryStatement clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryStatement copyWith(void Function(StoryStatement) updates) =>
      super.copyWith((message) => updates(message as StoryStatement))
          as StoryStatement;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StoryStatement() / StoryStatement.new instead')
  static StoryStatement create() => StoryStatement._();
  static $pb.GeneratedMessage $_createMessage() => StoryStatement._();
  @$core.override
  StoryStatement createEmptyInstance() => StoryStatement._();
  @$core.pragma('dart2js:noInline')
  static StoryStatement getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StoryStatement>(
          StoryStatement.$_createMessage);
  static StoryStatement? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  StoryStatement_Value whichValue() =>
      _StoryStatement_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  Narration get narration => $_getN(0);
  @$pb.TagNumber(1)
  set narration(Narration value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasNarration() => $_has(0);
  @$pb.TagNumber(1)
  void clearNarration() => $_clearField(1);
  @$pb.TagNumber(1)
  Narration ensureNarration() => $_ensure(0);

  @$pb.TagNumber(2)
  VariableOutput get variableOutput => $_getN(1);
  @$pb.TagNumber(2)
  set variableOutput(VariableOutput value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasVariableOutput() => $_has(1);
  @$pb.TagNumber(2)
  void clearVariableOutput() => $_clearField(2);
  @$pb.TagNumber(2)
  VariableOutput ensureVariableOutput() => $_ensure(1);

  @$pb.TagNumber(3)
  Dialogue get dialogue => $_getN(2);
  @$pb.TagNumber(3)
  set dialogue(Dialogue value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasDialogue() => $_has(2);
  @$pb.TagNumber(3)
  void clearDialogue() => $_clearField(3);
  @$pb.TagNumber(3)
  Dialogue ensureDialogue() => $_ensure(2);

  @$pb.TagNumber(4)
  StoryIfElse get ifElse => $_getN(3);
  @$pb.TagNumber(4)
  set ifElse(StoryIfElse value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasIfElse() => $_has(3);
  @$pb.TagNumber(4)
  void clearIfElse() => $_clearField(4);
  @$pb.TagNumber(4)
  StoryIfElse ensureIfElse() => $_ensure(3);

  @$pb.TagNumber(5)
  ChoiceBlock get choice => $_getN(4);
  @$pb.TagNumber(5)
  set choice(ChoiceBlock value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasChoice() => $_has(4);
  @$pb.TagNumber(5)
  void clearChoice() => $_clearField(5);
  @$pb.TagNumber(5)
  ChoiceBlock ensureChoice() => $_ensure(4);

  @$pb.TagNumber(6)
  Jump get jump => $_getN(5);
  @$pb.TagNumber(6)
  set jump(Jump value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasJump() => $_has(5);
  @$pb.TagNumber(6)
  void clearJump() => $_clearField(6);
  @$pb.TagNumber(6)
  Jump ensureJump() => $_ensure(5);

  @$pb.TagNumber(7)
  $0.Empty get end => $_getN(6);
  @$pb.TagNumber(7)
  set end($0.Empty value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEnd() => $_has(6);
  @$pb.TagNumber(7)
  void clearEnd() => $_clearField(7);
  @$pb.TagNumber(7)
  $0.Empty ensureEnd() => $_ensure(6);

  @$pb.TagNumber(8)
  SfxDirective get sfx => $_getN(7);
  @$pb.TagNumber(8)
  set sfx(SfxDirective value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasSfx() => $_has(7);
  @$pb.TagNumber(8)
  void clearSfx() => $_clearField(8);
  @$pb.TagNumber(8)
  SfxDirective ensureSfx() => $_ensure(7);

  @$pb.TagNumber(9)
  StoryForSnapshot get forSnapshot => $_getN(8);
  @$pb.TagNumber(9)
  set forSnapshot(StoryForSnapshot value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasForSnapshot() => $_has(8);
  @$pb.TagNumber(9)
  void clearForSnapshot() => $_clearField(9);
  @$pb.TagNumber(9)
  StoryForSnapshot ensureForSnapshot() => $_ensure(8);

  @$pb.TagNumber(10)
  StoryRepeat get repeat => $_getN(9);
  @$pb.TagNumber(10)
  set repeat(StoryRepeat value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasRepeat() => $_has(9);
  @$pb.TagNumber(10)
  void clearRepeat() => $_clearField(10);
  @$pb.TagNumber(10)
  StoryRepeat ensureRepeat() => $_ensure(9);

  @$pb.TagNumber(11)
  $0.Empty get breakLoop => $_getN(10);
  @$pb.TagNumber(11)
  set breakLoop($0.Empty value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasBreakLoop() => $_has(10);
  @$pb.TagNumber(11)
  void clearBreakLoop() => $_clearField(11);
  @$pb.TagNumber(11)
  $0.Empty ensureBreakLoop() => $_ensure(10);

  @$pb.TagNumber(12)
  $0.Empty get continueLoop => $_getN(11);
  @$pb.TagNumber(12)
  set continueLoop($0.Empty value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasContinueLoop() => $_has(11);
  @$pb.TagNumber(12)
  void clearContinueLoop() => $_clearField(12);
  @$pb.TagNumber(12)
  $0.Empty ensureContinueLoop() => $_ensure(11);
}

class Narration extends $pb.GeneratedMessage {
  factory Narration({
    InterpolatedString? text,
  }) {
    final result = Narration._();
    if (text != null) result.text = text;
    return result;
  }

  Narration._();

  factory Narration.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Narration()..mergeFromBuffer(data, registry);
  factory Narration.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Narration()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Narration',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Narration.$_createMessage)
    ..aOM<InterpolatedString>(1, _omitFieldNames ? '' : 'text',
        subBuilder: InterpolatedString.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Narration clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Narration copyWith(void Function(Narration) updates) =>
      super.copyWith((message) => updates(message as Narration)) as Narration;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Narration() / Narration.new instead')
  static Narration create() => Narration._();
  static $pb.GeneratedMessage $_createMessage() => Narration._();
  @$core.override
  Narration createEmptyInstance() => Narration._();
  @$core.pragma('dart2js:noInline')
  static Narration getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Narration>(Narration.$_createMessage);
  static Narration? _defaultInstance;

  @$pb.TagNumber(1)
  InterpolatedString get text => $_getN(0);
  @$pb.TagNumber(1)
  set text(InterpolatedString value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasText() => $_has(0);
  @$pb.TagNumber(1)
  void clearText() => $_clearField(1);
  @$pb.TagNumber(1)
  InterpolatedString ensureText() => $_ensure(0);
}

class VariableOutput extends $pb.GeneratedMessage {
  factory VariableOutput({
    $core.String? name,
  }) {
    final result = VariableOutput._();
    if (name != null) result.name = name;
    return result;
  }

  VariableOutput._();

  factory VariableOutput.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableOutput()..mergeFromBuffer(data, registry);
  factory VariableOutput.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableOutput()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VariableOutput',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: VariableOutput.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableOutput clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableOutput copyWith(void Function(VariableOutput) updates) =>
      super.copyWith((message) => updates(message as VariableOutput))
          as VariableOutput;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use VariableOutput() / VariableOutput.new instead')
  static VariableOutput create() => VariableOutput._();
  static $pb.GeneratedMessage $_createMessage() => VariableOutput._();
  @$core.override
  VariableOutput createEmptyInstance() => VariableOutput._();
  @$core.pragma('dart2js:noInline')
  static VariableOutput getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VariableOutput>(
          VariableOutput.$_createMessage);
  static VariableOutput? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

enum Dialogue_Form { nameOnly, portrait, notSet }

class Dialogue extends $pb.GeneratedMessage {
  factory Dialogue({
    $core.String? actorId,
    $0.Empty? nameOnly,
    PortraitDialogue? portrait,
    InterpolatedString? text,
  }) {
    final result = Dialogue._();
    if (actorId != null) result.actorId = actorId;
    if (nameOnly != null) result.nameOnly = nameOnly;
    if (portrait != null) result.portrait = portrait;
    if (text != null) result.text = text;
    return result;
  }

  Dialogue._();

  factory Dialogue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Dialogue()..mergeFromBuffer(data, registry);
  factory Dialogue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Dialogue()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Dialogue_Form> _Dialogue_FormByTag = {
    2: Dialogue_Form.nameOnly,
    3: Dialogue_Form.portrait,
    0: Dialogue_Form.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Dialogue',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Dialogue.$_createMessage)
    ..oo(0, [2, 3])
    ..aOS(1, _omitFieldNames ? '' : 'actorId')
    ..aOM<$0.Empty>(2, _omitFieldNames ? '' : 'nameOnly',
        subBuilder: $0.Empty.$_createMessage)
    ..aOM<PortraitDialogue>(3, _omitFieldNames ? '' : 'portrait',
        subBuilder: PortraitDialogue.$_createMessage)
    ..aOM<InterpolatedString>(4, _omitFieldNames ? '' : 'text',
        subBuilder: InterpolatedString.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Dialogue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Dialogue copyWith(void Function(Dialogue) updates) =>
      super.copyWith((message) => updates(message as Dialogue)) as Dialogue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Dialogue() / Dialogue.new instead')
  static Dialogue create() => Dialogue._();
  static $pb.GeneratedMessage $_createMessage() => Dialogue._();
  @$core.override
  Dialogue createEmptyInstance() => Dialogue._();
  @$core.pragma('dart2js:noInline')
  static Dialogue getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Dialogue>(Dialogue.$_createMessage);
  static Dialogue? _defaultInstance;

  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  Dialogue_Form whichForm() => _Dialogue_FormByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  void clearForm() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get actorId => $_getSZ(0);
  @$pb.TagNumber(1)
  set actorId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasActorId() => $_has(0);
  @$pb.TagNumber(1)
  void clearActorId() => $_clearField(1);

  @$pb.TagNumber(2)
  $0.Empty get nameOnly => $_getN(1);
  @$pb.TagNumber(2)
  set nameOnly($0.Empty value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasNameOnly() => $_has(1);
  @$pb.TagNumber(2)
  void clearNameOnly() => $_clearField(2);
  @$pb.TagNumber(2)
  $0.Empty ensureNameOnly() => $_ensure(1);

  @$pb.TagNumber(3)
  PortraitDialogue get portrait => $_getN(2);
  @$pb.TagNumber(3)
  set portrait(PortraitDialogue value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasPortrait() => $_has(2);
  @$pb.TagNumber(3)
  void clearPortrait() => $_clearField(3);
  @$pb.TagNumber(3)
  PortraitDialogue ensurePortrait() => $_ensure(2);

  @$pb.TagNumber(4)
  InterpolatedString get text => $_getN(3);
  @$pb.TagNumber(4)
  set text(InterpolatedString value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasText() => $_has(3);
  @$pb.TagNumber(4)
  void clearText() => $_clearField(4);
  @$pb.TagNumber(4)
  InterpolatedString ensureText() => $_ensure(3);
}

class PortraitDialogue extends $pb.GeneratedMessage {
  factory PortraitDialogue({
    $core.String? emotion,
    Position? position,
  }) {
    final result = PortraitDialogue._();
    if (emotion != null) result.emotion = emotion;
    if (position != null) result.position = position;
    return result;
  }

  PortraitDialogue._();

  factory PortraitDialogue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PortraitDialogue()..mergeFromBuffer(data, registry);
  factory PortraitDialogue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PortraitDialogue()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PortraitDialogue',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: PortraitDialogue.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'emotion')
    ..aE<Position>(2, _omitFieldNames ? '' : 'position',
        enumValues: Position.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PortraitDialogue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PortraitDialogue copyWith(void Function(PortraitDialogue) updates) =>
      super.copyWith((message) => updates(message as PortraitDialogue))
          as PortraitDialogue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PortraitDialogue() / PortraitDialogue.new instead')
  static PortraitDialogue create() => PortraitDialogue._();
  static $pb.GeneratedMessage $_createMessage() => PortraitDialogue._();
  @$core.override
  PortraitDialogue createEmptyInstance() => PortraitDialogue._();
  @$core.pragma('dart2js:noInline')
  static PortraitDialogue getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PortraitDialogue>(
          PortraitDialogue.$_createMessage);
  static PortraitDialogue? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get emotion => $_getSZ(0);
  @$pb.TagNumber(1)
  set emotion($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEmotion() => $_has(0);
  @$pb.TagNumber(1)
  void clearEmotion() => $_clearField(1);

  @$pb.TagNumber(2)
  Position get position => $_getN(1);
  @$pb.TagNumber(2)
  set position(Position value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPosition() => $_has(1);
  @$pb.TagNumber(2)
  void clearPosition() => $_clearField(2);
}

class StoryStatementList extends $pb.GeneratedMessage {
  factory StoryStatementList({
    $core.Iterable<StoryStatement>? statements,
  }) {
    final result = StoryStatementList._();
    if (statements != null) result.statements.addAll(statements);
    return result;
  }

  StoryStatementList._();

  factory StoryStatementList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryStatementList()..mergeFromBuffer(data, registry);
  factory StoryStatementList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryStatementList()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoryStatementList',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StoryStatementList.$_createMessage)
    ..pPM<StoryStatement>(1, _omitFieldNames ? '' : 'statements',
        subBuilder: StoryStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryStatementList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryStatementList copyWith(void Function(StoryStatementList) updates) =>
      super.copyWith((message) => updates(message as StoryStatementList))
          as StoryStatementList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StoryStatementList() / StoryStatementList.new instead')
  static StoryStatementList create() => StoryStatementList._();
  static $pb.GeneratedMessage $_createMessage() => StoryStatementList._();
  @$core.override
  StoryStatementList createEmptyInstance() => StoryStatementList._();
  @$core.pragma('dart2js:noInline')
  static StoryStatementList getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StoryStatementList>(
          StoryStatementList.$_createMessage);
  static StoryStatementList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<StoryStatement> get statements => $_getList(0);
}

class StoryIfElse extends $pb.GeneratedMessage {
  factory StoryIfElse({
    Expression? condition,
    StoryStatementList? thenBranch,
    StoryStatementList? elseBranch,
  }) {
    final result = StoryIfElse._();
    if (condition != null) result.condition = condition;
    if (thenBranch != null) result.thenBranch = thenBranch;
    if (elseBranch != null) result.elseBranch = elseBranch;
    return result;
  }

  StoryIfElse._();

  factory StoryIfElse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryIfElse()..mergeFromBuffer(data, registry);
  factory StoryIfElse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryIfElse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoryIfElse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StoryIfElse.$_createMessage)
    ..aOM<Expression>(1, _omitFieldNames ? '' : 'condition',
        subBuilder: Expression.$_createMessage)
    ..aOM<StoryStatementList>(2, _omitFieldNames ? '' : 'thenBranch',
        subBuilder: StoryStatementList.$_createMessage)
    ..aOM<StoryStatementList>(3, _omitFieldNames ? '' : 'elseBranch',
        subBuilder: StoryStatementList.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryIfElse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryIfElse copyWith(void Function(StoryIfElse) updates) =>
      super.copyWith((message) => updates(message as StoryIfElse))
          as StoryIfElse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StoryIfElse() / StoryIfElse.new instead')
  static StoryIfElse create() => StoryIfElse._();
  static $pb.GeneratedMessage $_createMessage() => StoryIfElse._();
  @$core.override
  StoryIfElse createEmptyInstance() => StoryIfElse._();
  @$core.pragma('dart2js:noInline')
  static StoryIfElse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StoryIfElse>(
          StoryIfElse.$_createMessage);
  static StoryIfElse? _defaultInstance;

  @$pb.TagNumber(1)
  Expression get condition => $_getN(0);
  @$pb.TagNumber(1)
  set condition(Expression value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCondition() => $_has(0);
  @$pb.TagNumber(1)
  void clearCondition() => $_clearField(1);
  @$pb.TagNumber(1)
  Expression ensureCondition() => $_ensure(0);

  @$pb.TagNumber(2)
  StoryStatementList get thenBranch => $_getN(1);
  @$pb.TagNumber(2)
  set thenBranch(StoryStatementList value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasThenBranch() => $_has(1);
  @$pb.TagNumber(2)
  void clearThenBranch() => $_clearField(2);
  @$pb.TagNumber(2)
  StoryStatementList ensureThenBranch() => $_ensure(1);

  @$pb.TagNumber(3)
  StoryStatementList get elseBranch => $_getN(2);
  @$pb.TagNumber(3)
  set elseBranch(StoryStatementList value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasElseBranch() => $_has(2);
  @$pb.TagNumber(3)
  void clearElseBranch() => $_clearField(3);
  @$pb.TagNumber(3)
  StoryStatementList ensureElseBranch() => $_ensure(2);
}

class ChoiceBlock extends $pb.GeneratedMessage {
  factory ChoiceBlock({
    $core.Iterable<ChoiceEntry>? entries,
  }) {
    final result = ChoiceBlock._();
    if (entries != null) result.entries.addAll(entries);
    return result;
  }

  ChoiceBlock._();

  factory ChoiceBlock.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceBlock()..mergeFromBuffer(data, registry);
  factory ChoiceBlock.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceBlock()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceBlock',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceBlock.$_createMessage)
    ..pPM<ChoiceEntry>(1, _omitFieldNames ? '' : 'entries',
        subBuilder: ChoiceEntry.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceBlock clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceBlock copyWith(void Function(ChoiceBlock) updates) =>
      super.copyWith((message) => updates(message as ChoiceBlock))
          as ChoiceBlock;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceBlock() / ChoiceBlock.new instead')
  static ChoiceBlock create() => ChoiceBlock._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceBlock._();
  @$core.override
  ChoiceBlock createEmptyInstance() => ChoiceBlock._();
  @$core.pragma('dart2js:noInline')
  static ChoiceBlock getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChoiceBlock>(
          ChoiceBlock.$_createMessage);
  static ChoiceBlock? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ChoiceEntry> get entries => $_getList(0);
}

enum ChoiceEntry_Value { option, ifEntry, repeat, forSnapshot, notSet }

class ChoiceEntry extends $pb.GeneratedMessage {
  factory ChoiceEntry({
    ChoiceOption? option,
    ChoiceIf? ifEntry,
    ChoiceRepeat? repeat,
    ChoiceForSnapshot? forSnapshot,
  }) {
    final result = ChoiceEntry._();
    if (option != null) result.option = option;
    if (ifEntry != null) result.ifEntry = ifEntry;
    if (repeat != null) result.repeat = repeat;
    if (forSnapshot != null) result.forSnapshot = forSnapshot;
    return result;
  }

  ChoiceEntry._();

  factory ChoiceEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceEntry()..mergeFromBuffer(data, registry);
  factory ChoiceEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceEntry()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ChoiceEntry_Value> _ChoiceEntry_ValueByTag =
      {
    1: ChoiceEntry_Value.option,
    2: ChoiceEntry_Value.ifEntry,
    3: ChoiceEntry_Value.repeat,
    4: ChoiceEntry_Value.forSnapshot,
    0: ChoiceEntry_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceEntry',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceEntry.$_createMessage)
    ..oo(0, [1, 2, 3, 4])
    ..aOM<ChoiceOption>(1, _omitFieldNames ? '' : 'option',
        subBuilder: ChoiceOption.$_createMessage)
    ..aOM<ChoiceIf>(2, _omitFieldNames ? '' : 'ifEntry',
        subBuilder: ChoiceIf.$_createMessage)
    ..aOM<ChoiceRepeat>(3, _omitFieldNames ? '' : 'repeat',
        subBuilder: ChoiceRepeat.$_createMessage)
    ..aOM<ChoiceForSnapshot>(4, _omitFieldNames ? '' : 'forSnapshot',
        subBuilder: ChoiceForSnapshot.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceEntry copyWith(void Function(ChoiceEntry) updates) =>
      super.copyWith((message) => updates(message as ChoiceEntry))
          as ChoiceEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceEntry() / ChoiceEntry.new instead')
  static ChoiceEntry create() => ChoiceEntry._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceEntry._();
  @$core.override
  ChoiceEntry createEmptyInstance() => ChoiceEntry._();
  @$core.pragma('dart2js:noInline')
  static ChoiceEntry getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChoiceEntry>(
          ChoiceEntry.$_createMessage);
  static ChoiceEntry? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  ChoiceEntry_Value whichValue() => _ChoiceEntry_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  ChoiceOption get option => $_getN(0);
  @$pb.TagNumber(1)
  set option(ChoiceOption value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasOption() => $_has(0);
  @$pb.TagNumber(1)
  void clearOption() => $_clearField(1);
  @$pb.TagNumber(1)
  ChoiceOption ensureOption() => $_ensure(0);

  @$pb.TagNumber(2)
  ChoiceIf get ifEntry => $_getN(1);
  @$pb.TagNumber(2)
  set ifEntry(ChoiceIf value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasIfEntry() => $_has(1);
  @$pb.TagNumber(2)
  void clearIfEntry() => $_clearField(2);
  @$pb.TagNumber(2)
  ChoiceIf ensureIfEntry() => $_ensure(1);

  @$pb.TagNumber(3)
  ChoiceRepeat get repeat => $_getN(2);
  @$pb.TagNumber(3)
  set repeat(ChoiceRepeat value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasRepeat() => $_has(2);
  @$pb.TagNumber(3)
  void clearRepeat() => $_clearField(3);
  @$pb.TagNumber(3)
  ChoiceRepeat ensureRepeat() => $_ensure(2);

  @$pb.TagNumber(4)
  ChoiceForSnapshot get forSnapshot => $_getN(3);
  @$pb.TagNumber(4)
  set forSnapshot(ChoiceForSnapshot value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasForSnapshot() => $_has(3);
  @$pb.TagNumber(4)
  void clearForSnapshot() => $_clearField(4);
  @$pb.TagNumber(4)
  ChoiceForSnapshot ensureForSnapshot() => $_ensure(3);
}

class ChoiceOption extends $pb.GeneratedMessage {
  factory ChoiceOption({
    InterpolatedString? text,
    $core.String? target,
  }) {
    final result = ChoiceOption._();
    if (text != null) result.text = text;
    if (target != null) result.target = target;
    return result;
  }

  ChoiceOption._();

  factory ChoiceOption.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceOption()..mergeFromBuffer(data, registry);
  factory ChoiceOption.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceOption()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceOption',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceOption.$_createMessage)
    ..aOM<InterpolatedString>(1, _omitFieldNames ? '' : 'text',
        subBuilder: InterpolatedString.$_createMessage)
    ..aOS(2, _omitFieldNames ? '' : 'target')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceOption clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceOption copyWith(void Function(ChoiceOption) updates) =>
      super.copyWith((message) => updates(message as ChoiceOption))
          as ChoiceOption;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceOption() / ChoiceOption.new instead')
  static ChoiceOption create() => ChoiceOption._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceOption._();
  @$core.override
  ChoiceOption createEmptyInstance() => ChoiceOption._();
  @$core.pragma('dart2js:noInline')
  static ChoiceOption getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChoiceOption>(
          ChoiceOption.$_createMessage);
  static ChoiceOption? _defaultInstance;

  @$pb.TagNumber(1)
  InterpolatedString get text => $_getN(0);
  @$pb.TagNumber(1)
  set text(InterpolatedString value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasText() => $_has(0);
  @$pb.TagNumber(1)
  void clearText() => $_clearField(1);
  @$pb.TagNumber(1)
  InterpolatedString ensureText() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get target => $_getSZ(1);
  @$pb.TagNumber(2)
  set target($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTarget() => $_has(1);
  @$pb.TagNumber(2)
  void clearTarget() => $_clearField(2);
}

class ChoiceEntryList extends $pb.GeneratedMessage {
  factory ChoiceEntryList({
    $core.Iterable<ChoiceEntry>? entries,
  }) {
    final result = ChoiceEntryList._();
    if (entries != null) result.entries.addAll(entries);
    return result;
  }

  ChoiceEntryList._();

  factory ChoiceEntryList.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceEntryList()..mergeFromBuffer(data, registry);
  factory ChoiceEntryList.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceEntryList()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceEntryList',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceEntryList.$_createMessage)
    ..pPM<ChoiceEntry>(1, _omitFieldNames ? '' : 'entries',
        subBuilder: ChoiceEntry.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceEntryList clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceEntryList copyWith(void Function(ChoiceEntryList) updates) =>
      super.copyWith((message) => updates(message as ChoiceEntryList))
          as ChoiceEntryList;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceEntryList() / ChoiceEntryList.new instead')
  static ChoiceEntryList create() => ChoiceEntryList._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceEntryList._();
  @$core.override
  ChoiceEntryList createEmptyInstance() => ChoiceEntryList._();
  @$core.pragma('dart2js:noInline')
  static ChoiceEntryList getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChoiceEntryList>(
          ChoiceEntryList.$_createMessage);
  static ChoiceEntryList? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ChoiceEntry> get entries => $_getList(0);
}

class ChoiceIf extends $pb.GeneratedMessage {
  factory ChoiceIf({
    Expression? condition,
    ChoiceEntryList? body,
  }) {
    final result = ChoiceIf._();
    if (condition != null) result.condition = condition;
    if (body != null) result.body = body;
    return result;
  }

  ChoiceIf._();

  factory ChoiceIf.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceIf()..mergeFromBuffer(data, registry);
  factory ChoiceIf.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceIf()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceIf',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceIf.$_createMessage)
    ..aOM<Expression>(1, _omitFieldNames ? '' : 'condition',
        subBuilder: Expression.$_createMessage)
    ..aOM<ChoiceEntryList>(2, _omitFieldNames ? '' : 'body',
        subBuilder: ChoiceEntryList.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceIf clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceIf copyWith(void Function(ChoiceIf) updates) =>
      super.copyWith((message) => updates(message as ChoiceIf)) as ChoiceIf;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceIf() / ChoiceIf.new instead')
  static ChoiceIf create() => ChoiceIf._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceIf._();
  @$core.override
  ChoiceIf createEmptyInstance() => ChoiceIf._();
  @$core.pragma('dart2js:noInline')
  static ChoiceIf getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ChoiceIf>(ChoiceIf.$_createMessage);
  static ChoiceIf? _defaultInstance;

  @$pb.TagNumber(1)
  Expression get condition => $_getN(0);
  @$pb.TagNumber(1)
  set condition(Expression value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCondition() => $_has(0);
  @$pb.TagNumber(1)
  void clearCondition() => $_clearField(1);
  @$pb.TagNumber(1)
  Expression ensureCondition() => $_ensure(0);

  @$pb.TagNumber(2)
  ChoiceEntryList get body => $_getN(1);
  @$pb.TagNumber(2)
  set body(ChoiceEntryList value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasBody() => $_has(1);
  @$pb.TagNumber(2)
  void clearBody() => $_clearField(2);
  @$pb.TagNumber(2)
  ChoiceEntryList ensureBody() => $_ensure(1);
}

class ChoiceRepeat extends $pb.GeneratedMessage {
  factory ChoiceRepeat({
    RepeatCount? count,
    ChoiceEntryList? body,
  }) {
    final result = ChoiceRepeat._();
    if (count != null) result.count = count;
    if (body != null) result.body = body;
    return result;
  }

  ChoiceRepeat._();

  factory ChoiceRepeat.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceRepeat()..mergeFromBuffer(data, registry);
  factory ChoiceRepeat.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceRepeat()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceRepeat',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceRepeat.$_createMessage)
    ..aOM<RepeatCount>(1, _omitFieldNames ? '' : 'count',
        subBuilder: RepeatCount.$_createMessage)
    ..aOM<ChoiceEntryList>(2, _omitFieldNames ? '' : 'body',
        subBuilder: ChoiceEntryList.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceRepeat clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceRepeat copyWith(void Function(ChoiceRepeat) updates) =>
      super.copyWith((message) => updates(message as ChoiceRepeat))
          as ChoiceRepeat;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceRepeat() / ChoiceRepeat.new instead')
  static ChoiceRepeat create() => ChoiceRepeat._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceRepeat._();
  @$core.override
  ChoiceRepeat createEmptyInstance() => ChoiceRepeat._();
  @$core.pragma('dart2js:noInline')
  static ChoiceRepeat getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChoiceRepeat>(
          ChoiceRepeat.$_createMessage);
  static ChoiceRepeat? _defaultInstance;

  @$pb.TagNumber(1)
  RepeatCount get count => $_getN(0);
  @$pb.TagNumber(1)
  set count(RepeatCount value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => $_clearField(1);
  @$pb.TagNumber(1)
  RepeatCount ensureCount() => $_ensure(0);

  @$pb.TagNumber(2)
  ChoiceEntryList get body => $_getN(1);
  @$pb.TagNumber(2)
  set body(ChoiceEntryList value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasBody() => $_has(1);
  @$pb.TagNumber(2)
  void clearBody() => $_clearField(2);
  @$pb.TagNumber(2)
  ChoiceEntryList ensureBody() => $_ensure(1);
}

class ChoiceForSnapshot extends $pb.GeneratedMessage {
  factory ChoiceForSnapshot({
    $core.String? itemName,
    $core.String? arrayName,
    ChoiceEntryList? body,
  }) {
    final result = ChoiceForSnapshot._();
    if (itemName != null) result.itemName = itemName;
    if (arrayName != null) result.arrayName = arrayName;
    if (body != null) result.body = body;
    return result;
  }

  ChoiceForSnapshot._();

  factory ChoiceForSnapshot.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceForSnapshot()..mergeFromBuffer(data, registry);
  factory ChoiceForSnapshot.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ChoiceForSnapshot()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ChoiceForSnapshot',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ChoiceForSnapshot.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'itemName')
    ..aOS(2, _omitFieldNames ? '' : 'arrayName')
    ..aOM<ChoiceEntryList>(3, _omitFieldNames ? '' : 'body',
        subBuilder: ChoiceEntryList.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceForSnapshot clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ChoiceForSnapshot copyWith(void Function(ChoiceForSnapshot) updates) =>
      super.copyWith((message) => updates(message as ChoiceForSnapshot))
          as ChoiceForSnapshot;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ChoiceForSnapshot() / ChoiceForSnapshot.new instead')
  static ChoiceForSnapshot create() => ChoiceForSnapshot._();
  static $pb.GeneratedMessage $_createMessage() => ChoiceForSnapshot._();
  @$core.override
  ChoiceForSnapshot createEmptyInstance() => ChoiceForSnapshot._();
  @$core.pragma('dart2js:noInline')
  static ChoiceForSnapshot getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ChoiceForSnapshot>(
          ChoiceForSnapshot.$_createMessage);
  static ChoiceForSnapshot? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemName => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemName() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get arrayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set arrayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasArrayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearArrayName() => $_clearField(2);

  @$pb.TagNumber(3)
  ChoiceEntryList get body => $_getN(2);
  @$pb.TagNumber(3)
  set body(ChoiceEntryList value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasBody() => $_has(2);
  @$pb.TagNumber(3)
  void clearBody() => $_clearField(3);
  @$pb.TagNumber(3)
  ChoiceEntryList ensureBody() => $_ensure(2);
}

class Jump extends $pb.GeneratedMessage {
  factory Jump({
    $core.String? target,
  }) {
    final result = Jump._();
    if (target != null) result.target = target;
    return result;
  }

  Jump._();

  factory Jump.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Jump()..mergeFromBuffer(data, registry);
  factory Jump.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Jump()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Jump',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Jump.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'target')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Jump clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Jump copyWith(void Function(Jump) updates) =>
      super.copyWith((message) => updates(message as Jump)) as Jump;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Jump() / Jump.new instead')
  static Jump create() => Jump._();
  static $pb.GeneratedMessage $_createMessage() => Jump._();
  @$core.override
  Jump createEmptyInstance() => Jump._();
  @$core.pragma('dart2js:noInline')
  static Jump getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Jump>(Jump.$_createMessage);
  static Jump? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get target => $_getSZ(0);
  @$pb.TagNumber(1)
  set target($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);
}

class StoryForSnapshot extends $pb.GeneratedMessage {
  factory StoryForSnapshot({
    $core.String? itemName,
    $core.String? arrayName,
    $core.Iterable<StoryStatement>? body,
  }) {
    final result = StoryForSnapshot._();
    if (itemName != null) result.itemName = itemName;
    if (arrayName != null) result.arrayName = arrayName;
    if (body != null) result.body.addAll(body);
    return result;
  }

  StoryForSnapshot._();

  factory StoryForSnapshot.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryForSnapshot()..mergeFromBuffer(data, registry);
  factory StoryForSnapshot.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryForSnapshot()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoryForSnapshot',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StoryForSnapshot.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'itemName')
    ..aOS(2, _omitFieldNames ? '' : 'arrayName')
    ..pPM<StoryStatement>(3, _omitFieldNames ? '' : 'body',
        subBuilder: StoryStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryForSnapshot clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryForSnapshot copyWith(void Function(StoryForSnapshot) updates) =>
      super.copyWith((message) => updates(message as StoryForSnapshot))
          as StoryForSnapshot;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StoryForSnapshot() / StoryForSnapshot.new instead')
  static StoryForSnapshot create() => StoryForSnapshot._();
  static $pb.GeneratedMessage $_createMessage() => StoryForSnapshot._();
  @$core.override
  StoryForSnapshot createEmptyInstance() => StoryForSnapshot._();
  @$core.pragma('dart2js:noInline')
  static StoryForSnapshot getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StoryForSnapshot>(
          StoryForSnapshot.$_createMessage);
  static StoryForSnapshot? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemName => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemName() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get arrayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set arrayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasArrayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearArrayName() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<StoryStatement> get body => $_getList(2);
}

class StoryRepeat extends $pb.GeneratedMessage {
  factory StoryRepeat({
    RepeatCount? count,
    $core.Iterable<StoryStatement>? body,
  }) {
    final result = StoryRepeat._();
    if (count != null) result.count = count;
    if (body != null) result.body.addAll(body);
    return result;
  }

  StoryRepeat._();

  factory StoryRepeat.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryRepeat()..mergeFromBuffer(data, registry);
  factory StoryRepeat.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StoryRepeat()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoryRepeat',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: StoryRepeat.$_createMessage)
    ..aOM<RepeatCount>(1, _omitFieldNames ? '' : 'count',
        subBuilder: RepeatCount.$_createMessage)
    ..pPM<StoryStatement>(2, _omitFieldNames ? '' : 'body',
        subBuilder: StoryStatement.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryRepeat clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoryRepeat copyWith(void Function(StoryRepeat) updates) =>
      super.copyWith((message) => updates(message as StoryRepeat))
          as StoryRepeat;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use StoryRepeat() / StoryRepeat.new instead')
  static StoryRepeat create() => StoryRepeat._();
  static $pb.GeneratedMessage $_createMessage() => StoryRepeat._();
  @$core.override
  StoryRepeat createEmptyInstance() => StoryRepeat._();
  @$core.pragma('dart2js:noInline')
  static StoryRepeat getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StoryRepeat>(
          StoryRepeat.$_createMessage);
  static StoryRepeat? _defaultInstance;

  @$pb.TagNumber(1)
  RepeatCount get count => $_getN(0);
  @$pb.TagNumber(1)
  set count(RepeatCount value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => $_clearField(1);
  @$pb.TagNumber(1)
  RepeatCount ensureCount() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<StoryStatement> get body => $_getList(1);
}

enum RepeatCount_Value { integer, variable, notSet }

class RepeatCount extends $pb.GeneratedMessage {
  factory RepeatCount({
    $fixnum.Int64? integer,
    $core.String? variable,
  }) {
    final result = RepeatCount._();
    if (integer != null) result.integer = integer;
    if (variable != null) result.variable = variable;
    return result;
  }

  RepeatCount._();

  factory RepeatCount.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RepeatCount()..mergeFromBuffer(data, registry);
  factory RepeatCount.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RepeatCount()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, RepeatCount_Value> _RepeatCount_ValueByTag =
      {
    1: RepeatCount_Value.integer,
    2: RepeatCount_Value.variable,
    0: RepeatCount_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RepeatCount',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: RepeatCount.$_createMessage)
    ..oo(0, [1, 2])
    ..aInt64(1, _omitFieldNames ? '' : 'integer')
    ..aOS(2, _omitFieldNames ? '' : 'variable')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RepeatCount clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RepeatCount copyWith(void Function(RepeatCount) updates) =>
      super.copyWith((message) => updates(message as RepeatCount))
          as RepeatCount;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RepeatCount() / RepeatCount.new instead')
  static RepeatCount create() => RepeatCount._();
  static $pb.GeneratedMessage $_createMessage() => RepeatCount._();
  @$core.override
  RepeatCount createEmptyInstance() => RepeatCount._();
  @$core.pragma('dart2js:noInline')
  static RepeatCount getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RepeatCount>(
          RepeatCount.$_createMessage);
  static RepeatCount? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  RepeatCount_Value whichValue() => _RepeatCount_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get integer => $_getI64(0);
  @$pb.TagNumber(1)
  set integer($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInteger() => $_has(0);
  @$pb.TagNumber(1)
  void clearInteger() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get variable => $_getSZ(1);
  @$pb.TagNumber(2)
  set variable($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVariable() => $_has(1);
  @$pb.TagNumber(2)
  void clearVariable() => $_clearField(2);
}

enum Expression_Value {
  integer,
  decimal,
  boolean,
  stringValue,
  variable,
  binary,
  call,
  list,
  notSet
}

class Expression extends $pb.GeneratedMessage {
  factory Expression({
    $fixnum.Int64? integer,
    DecimalValue? decimal,
    $core.bool? boolean,
    InterpolatedString? stringValue,
    VariableReference? variable,
    BinaryExpression? binary,
    CallExpression? call,
    ListExpression? list,
  }) {
    final result = Expression._();
    if (integer != null) result.integer = integer;
    if (decimal != null) result.decimal = decimal;
    if (boolean != null) result.boolean = boolean;
    if (stringValue != null) result.stringValue = stringValue;
    if (variable != null) result.variable = variable;
    if (binary != null) result.binary = binary;
    if (call != null) result.call = call;
    if (list != null) result.list = list;
    return result;
  }

  Expression._();

  factory Expression.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Expression()..mergeFromBuffer(data, registry);
  factory Expression.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Expression()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Expression_Value> _Expression_ValueByTag = {
    1: Expression_Value.integer,
    2: Expression_Value.decimal,
    3: Expression_Value.boolean,
    4: Expression_Value.stringValue,
    5: Expression_Value.variable,
    6: Expression_Value.binary,
    7: Expression_Value.call,
    8: Expression_Value.list,
    0: Expression_Value.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Expression',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: Expression.$_createMessage)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8])
    ..aInt64(1, _omitFieldNames ? '' : 'integer')
    ..aOM<DecimalValue>(2, _omitFieldNames ? '' : 'decimal',
        subBuilder: DecimalValue.$_createMessage)
    ..aOB(3, _omitFieldNames ? '' : 'boolean')
    ..aOM<InterpolatedString>(4, _omitFieldNames ? '' : 'stringValue',
        subBuilder: InterpolatedString.$_createMessage)
    ..aOM<VariableReference>(5, _omitFieldNames ? '' : 'variable',
        subBuilder: VariableReference.$_createMessage)
    ..aOM<BinaryExpression>(6, _omitFieldNames ? '' : 'binary',
        subBuilder: BinaryExpression.$_createMessage)
    ..aOM<CallExpression>(7, _omitFieldNames ? '' : 'call',
        subBuilder: CallExpression.$_createMessage)
    ..aOM<ListExpression>(8, _omitFieldNames ? '' : 'list',
        subBuilder: ListExpression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Expression clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Expression copyWith(void Function(Expression) updates) =>
      super.copyWith((message) => updates(message as Expression)) as Expression;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Expression() / Expression.new instead')
  static Expression create() => Expression._();
  static $pb.GeneratedMessage $_createMessage() => Expression._();
  @$core.override
  Expression createEmptyInstance() => Expression._();
  @$core.pragma('dart2js:noInline')
  static Expression getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Expression>(Expression.$_createMessage);
  static Expression? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  Expression_Value whichValue() => _Expression_ValueByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  void clearValue() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get integer => $_getI64(0);
  @$pb.TagNumber(1)
  set integer($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInteger() => $_has(0);
  @$pb.TagNumber(1)
  void clearInteger() => $_clearField(1);

  @$pb.TagNumber(2)
  DecimalValue get decimal => $_getN(1);
  @$pb.TagNumber(2)
  set decimal(DecimalValue value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasDecimal() => $_has(1);
  @$pb.TagNumber(2)
  void clearDecimal() => $_clearField(2);
  @$pb.TagNumber(2)
  DecimalValue ensureDecimal() => $_ensure(1);

  @$pb.TagNumber(3)
  $core.bool get boolean => $_getBF(2);
  @$pb.TagNumber(3)
  set boolean($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasBoolean() => $_has(2);
  @$pb.TagNumber(3)
  void clearBoolean() => $_clearField(3);

  @$pb.TagNumber(4)
  InterpolatedString get stringValue => $_getN(3);
  @$pb.TagNumber(4)
  set stringValue(InterpolatedString value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasStringValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearStringValue() => $_clearField(4);
  @$pb.TagNumber(4)
  InterpolatedString ensureStringValue() => $_ensure(3);

  @$pb.TagNumber(5)
  VariableReference get variable => $_getN(4);
  @$pb.TagNumber(5)
  set variable(VariableReference value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasVariable() => $_has(4);
  @$pb.TagNumber(5)
  void clearVariable() => $_clearField(5);
  @$pb.TagNumber(5)
  VariableReference ensureVariable() => $_ensure(4);

  @$pb.TagNumber(6)
  BinaryExpression get binary => $_getN(5);
  @$pb.TagNumber(6)
  set binary(BinaryExpression value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasBinary() => $_has(5);
  @$pb.TagNumber(6)
  void clearBinary() => $_clearField(6);
  @$pb.TagNumber(6)
  BinaryExpression ensureBinary() => $_ensure(5);

  @$pb.TagNumber(7)
  CallExpression get call => $_getN(6);
  @$pb.TagNumber(7)
  set call(CallExpression value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasCall() => $_has(6);
  @$pb.TagNumber(7)
  void clearCall() => $_clearField(7);
  @$pb.TagNumber(7)
  CallExpression ensureCall() => $_ensure(6);

  @$pb.TagNumber(8)
  ListExpression get list => $_getN(7);
  @$pb.TagNumber(8)
  set list(ListExpression value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasList() => $_has(7);
  @$pb.TagNumber(8)
  void clearList() => $_clearField(8);
  @$pb.TagNumber(8)
  ListExpression ensureList() => $_ensure(7);
}

/// Base-10 representation with no exponent. Scale is significant: 1.0 and
/// 1.00 are distinct canonical values and must round-trip unchanged.
class DecimalValue extends $pb.GeneratedMessage {
  factory DecimalValue({
    $core.String? canonical,
  }) {
    final result = DecimalValue._();
    if (canonical != null) result.canonical = canonical;
    return result;
  }

  DecimalValue._();

  factory DecimalValue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DecimalValue()..mergeFromBuffer(data, registry);
  factory DecimalValue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DecimalValue()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DecimalValue',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: DecimalValue.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'canonical')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DecimalValue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DecimalValue copyWith(void Function(DecimalValue) updates) =>
      super.copyWith((message) => updates(message as DecimalValue))
          as DecimalValue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DecimalValue() / DecimalValue.new instead')
  static DecimalValue create() => DecimalValue._();
  static $pb.GeneratedMessage $_createMessage() => DecimalValue._();
  @$core.override
  DecimalValue createEmptyInstance() => DecimalValue._();
  @$core.pragma('dart2js:noInline')
  static DecimalValue getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DecimalValue>(
          DecimalValue.$_createMessage);
  static DecimalValue? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get canonical => $_getSZ(0);
  @$pb.TagNumber(1)
  set canonical($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCanonical() => $_has(0);
  @$pb.TagNumber(1)
  void clearCanonical() => $_clearField(1);
}

class VariableReference extends $pb.GeneratedMessage {
  factory VariableReference({
    $core.String? name,
  }) {
    final result = VariableReference._();
    if (name != null) result.name = name;
    return result;
  }

  VariableReference._();

  factory VariableReference.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableReference()..mergeFromBuffer(data, registry);
  factory VariableReference.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VariableReference()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VariableReference',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: VariableReference.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableReference clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VariableReference copyWith(void Function(VariableReference) updates) =>
      super.copyWith((message) => updates(message as VariableReference))
          as VariableReference;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use VariableReference() / VariableReference.new instead')
  static VariableReference create() => VariableReference._();
  static $pb.GeneratedMessage $_createMessage() => VariableReference._();
  @$core.override
  VariableReference createEmptyInstance() => VariableReference._();
  @$core.pragma('dart2js:noInline')
  static VariableReference getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VariableReference>(
          VariableReference.$_createMessage);
  static VariableReference? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

class BinaryExpression extends $pb.GeneratedMessage {
  factory BinaryExpression({
    Expression? left,
    BinaryOperator? operator,
    Expression? right,
  }) {
    final result = BinaryExpression._();
    if (left != null) result.left = left;
    if (operator != null) result.operator = operator;
    if (right != null) result.right = right;
    return result;
  }

  BinaryExpression._();

  factory BinaryExpression.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BinaryExpression()..mergeFromBuffer(data, registry);
  factory BinaryExpression.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      BinaryExpression()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BinaryExpression',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: BinaryExpression.$_createMessage)
    ..aOM<Expression>(1, _omitFieldNames ? '' : 'left',
        subBuilder: Expression.$_createMessage)
    ..aE<BinaryOperator>(2, _omitFieldNames ? '' : 'operator',
        enumValues: BinaryOperator.values)
    ..aOM<Expression>(3, _omitFieldNames ? '' : 'right',
        subBuilder: Expression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BinaryExpression clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BinaryExpression copyWith(void Function(BinaryExpression) updates) =>
      super.copyWith((message) => updates(message as BinaryExpression))
          as BinaryExpression;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use BinaryExpression() / BinaryExpression.new instead')
  static BinaryExpression create() => BinaryExpression._();
  static $pb.GeneratedMessage $_createMessage() => BinaryExpression._();
  @$core.override
  BinaryExpression createEmptyInstance() => BinaryExpression._();
  @$core.pragma('dart2js:noInline')
  static BinaryExpression getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<BinaryExpression>(
          BinaryExpression.$_createMessage);
  static BinaryExpression? _defaultInstance;

  @$pb.TagNumber(1)
  Expression get left => $_getN(0);
  @$pb.TagNumber(1)
  set left(Expression value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasLeft() => $_has(0);
  @$pb.TagNumber(1)
  void clearLeft() => $_clearField(1);
  @$pb.TagNumber(1)
  Expression ensureLeft() => $_ensure(0);

  @$pb.TagNumber(2)
  BinaryOperator get operator => $_getN(1);
  @$pb.TagNumber(2)
  set operator(BinaryOperator value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasOperator() => $_has(1);
  @$pb.TagNumber(2)
  void clearOperator() => $_clearField(2);

  @$pb.TagNumber(3)
  Expression get right => $_getN(2);
  @$pb.TagNumber(3)
  set right(Expression value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasRight() => $_has(2);
  @$pb.TagNumber(3)
  void clearRight() => $_clearField(3);
  @$pb.TagNumber(3)
  Expression ensureRight() => $_ensure(2);
}

class CallExpression extends $pb.GeneratedMessage {
  factory CallExpression({
    $core.String? name,
    $core.Iterable<Expression>? arguments,
  }) {
    final result = CallExpression._();
    if (name != null) result.name = name;
    if (arguments != null) result.arguments.addAll(arguments);
    return result;
  }

  CallExpression._();

  factory CallExpression.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CallExpression()..mergeFromBuffer(data, registry);
  factory CallExpression.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CallExpression()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CallExpression',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: CallExpression.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..pPM<Expression>(2, _omitFieldNames ? '' : 'arguments',
        subBuilder: Expression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CallExpression clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CallExpression copyWith(void Function(CallExpression) updates) =>
      super.copyWith((message) => updates(message as CallExpression))
          as CallExpression;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CallExpression() / CallExpression.new instead')
  static CallExpression create() => CallExpression._();
  static $pb.GeneratedMessage $_createMessage() => CallExpression._();
  @$core.override
  CallExpression createEmptyInstance() => CallExpression._();
  @$core.pragma('dart2js:noInline')
  static CallExpression getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CallExpression>(
          CallExpression.$_createMessage);
  static CallExpression? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<Expression> get arguments => $_getList(1);
}

class ListExpression extends $pb.GeneratedMessage {
  factory ListExpression({
    $core.Iterable<Expression>? items,
  }) {
    final result = ListExpression._();
    if (items != null) result.items.addAll(items);
    return result;
  }

  ListExpression._();

  factory ListExpression.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListExpression()..mergeFromBuffer(data, registry);
  factory ListExpression.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListExpression()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListExpression',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'storybundle.v1'),
      createEmptyInstance: ListExpression.$_createMessage)
    ..pPM<Expression>(1, _omitFieldNames ? '' : 'items',
        subBuilder: Expression.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListExpression clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListExpression copyWith(void Function(ListExpression) updates) =>
      super.copyWith((message) => updates(message as ListExpression))
          as ListExpression;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListExpression() / ListExpression.new instead')
  static ListExpression create() => ListExpression._();
  static $pb.GeneratedMessage $_createMessage() => ListExpression._();
  @$core.override
  ListExpression createEmptyInstance() => ListExpression._();
  @$core.pragma('dart2js:noInline')
  static ListExpression getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListExpression>(
          ListExpression.$_createMessage);
  static ListExpression? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Expression> get items => $_getList(0);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
