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

import 'package:protobuf/protobuf.dart' as $pb;

class VariableType extends $pb.ProtobufEnum {
  static const VariableType VARIABLE_TYPE_UNSPECIFIED =
      VariableType._(0, _omitEnumNames ? '' : 'VARIABLE_TYPE_UNSPECIFIED');
  static const VariableType VARIABLE_TYPE_INTEGER =
      VariableType._(1, _omitEnumNames ? '' : 'VARIABLE_TYPE_INTEGER');
  static const VariableType VARIABLE_TYPE_STRING =
      VariableType._(2, _omitEnumNames ? '' : 'VARIABLE_TYPE_STRING');
  static const VariableType VARIABLE_TYPE_BOOLEAN =
      VariableType._(3, _omitEnumNames ? '' : 'VARIABLE_TYPE_BOOLEAN');
  static const VariableType VARIABLE_TYPE_DECIMAL =
      VariableType._(4, _omitEnumNames ? '' : 'VARIABLE_TYPE_DECIMAL');
  static const VariableType VARIABLE_TYPE_ARRAY_INTEGER =
      VariableType._(5, _omitEnumNames ? '' : 'VARIABLE_TYPE_ARRAY_INTEGER');
  static const VariableType VARIABLE_TYPE_ARRAY_STRING =
      VariableType._(6, _omitEnumNames ? '' : 'VARIABLE_TYPE_ARRAY_STRING');
  static const VariableType VARIABLE_TYPE_ARRAY_BOOLEAN =
      VariableType._(7, _omitEnumNames ? '' : 'VARIABLE_TYPE_ARRAY_BOOLEAN');
  static const VariableType VARIABLE_TYPE_ARRAY_DECIMAL =
      VariableType._(8, _omitEnumNames ? '' : 'VARIABLE_TYPE_ARRAY_DECIMAL');

  static const $core.List<VariableType> values = <VariableType>[
    VARIABLE_TYPE_UNSPECIFIED,
    VARIABLE_TYPE_INTEGER,
    VARIABLE_TYPE_STRING,
    VARIABLE_TYPE_BOOLEAN,
    VARIABLE_TYPE_DECIMAL,
    VARIABLE_TYPE_ARRAY_INTEGER,
    VARIABLE_TYPE_ARRAY_STRING,
    VARIABLE_TYPE_ARRAY_BOOLEAN,
    VARIABLE_TYPE_ARRAY_DECIMAL,
  ];

  static final $core.List<VariableType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 8);
  static VariableType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const VariableType._(super.value, super.name);
}

class AssignmentOperator extends $pb.ProtobufEnum {
  static const AssignmentOperator ASSIGNMENT_OPERATOR_UNSPECIFIED =
      AssignmentOperator._(
          0, _omitEnumNames ? '' : 'ASSIGNMENT_OPERATOR_UNSPECIFIED');
  static const AssignmentOperator ASSIGNMENT_OPERATOR_SET =
      AssignmentOperator._(1, _omitEnumNames ? '' : 'ASSIGNMENT_OPERATOR_SET');
  static const AssignmentOperator ASSIGNMENT_OPERATOR_ADD =
      AssignmentOperator._(2, _omitEnumNames ? '' : 'ASSIGNMENT_OPERATOR_ADD');
  static const AssignmentOperator ASSIGNMENT_OPERATOR_SUBTRACT =
      AssignmentOperator._(
          3, _omitEnumNames ? '' : 'ASSIGNMENT_OPERATOR_SUBTRACT');

  static const $core.List<AssignmentOperator> values = <AssignmentOperator>[
    ASSIGNMENT_OPERATOR_UNSPECIFIED,
    ASSIGNMENT_OPERATOR_SET,
    ASSIGNMENT_OPERATOR_ADD,
    ASSIGNMENT_OPERATOR_SUBTRACT,
  ];

  static final $core.List<AssignmentOperator?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static AssignmentOperator? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const AssignmentOperator._(super.value, super.name);
}

class Position extends $pb.ProtobufEnum {
  static const Position POSITION_UNSPECIFIED =
      Position._(0, _omitEnumNames ? '' : 'POSITION_UNSPECIFIED');
  static const Position POSITION_LEFT =
      Position._(1, _omitEnumNames ? '' : 'POSITION_LEFT');
  static const Position POSITION_CENTER =
      Position._(2, _omitEnumNames ? '' : 'POSITION_CENTER');
  static const Position POSITION_RIGHT =
      Position._(3, _omitEnumNames ? '' : 'POSITION_RIGHT');

  static const $core.List<Position> values = <Position>[
    POSITION_UNSPECIFIED,
    POSITION_LEFT,
    POSITION_CENTER,
    POSITION_RIGHT,
  ];

  static final $core.List<Position?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static Position? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const Position._(super.value, super.name);
}

class BinaryOperator extends $pb.ProtobufEnum {
  static const BinaryOperator BINARY_OPERATOR_UNSPECIFIED =
      BinaryOperator._(0, _omitEnumNames ? '' : 'BINARY_OPERATOR_UNSPECIFIED');
  static const BinaryOperator BINARY_OPERATOR_ADD =
      BinaryOperator._(1, _omitEnumNames ? '' : 'BINARY_OPERATOR_ADD');
  static const BinaryOperator BINARY_OPERATOR_SUBTRACT =
      BinaryOperator._(2, _omitEnumNames ? '' : 'BINARY_OPERATOR_SUBTRACT');
  static const BinaryOperator BINARY_OPERATOR_MULTIPLY =
      BinaryOperator._(3, _omitEnumNames ? '' : 'BINARY_OPERATOR_MULTIPLY');
  static const BinaryOperator BINARY_OPERATOR_DIVIDE =
      BinaryOperator._(4, _omitEnumNames ? '' : 'BINARY_OPERATOR_DIVIDE');
  static const BinaryOperator BINARY_OPERATOR_MODULO =
      BinaryOperator._(5, _omitEnumNames ? '' : 'BINARY_OPERATOR_MODULO');
  static const BinaryOperator BINARY_OPERATOR_EQUAL =
      BinaryOperator._(6, _omitEnumNames ? '' : 'BINARY_OPERATOR_EQUAL');
  static const BinaryOperator BINARY_OPERATOR_NOT_EQUAL =
      BinaryOperator._(7, _omitEnumNames ? '' : 'BINARY_OPERATOR_NOT_EQUAL');
  static const BinaryOperator BINARY_OPERATOR_LESS =
      BinaryOperator._(8, _omitEnumNames ? '' : 'BINARY_OPERATOR_LESS');
  static const BinaryOperator BINARY_OPERATOR_LESS_EQUAL =
      BinaryOperator._(9, _omitEnumNames ? '' : 'BINARY_OPERATOR_LESS_EQUAL');
  static const BinaryOperator BINARY_OPERATOR_GREATER =
      BinaryOperator._(10, _omitEnumNames ? '' : 'BINARY_OPERATOR_GREATER');
  static const BinaryOperator BINARY_OPERATOR_GREATER_EQUAL = BinaryOperator._(
      11, _omitEnumNames ? '' : 'BINARY_OPERATOR_GREATER_EQUAL');

  static const $core.List<BinaryOperator> values = <BinaryOperator>[
    BINARY_OPERATOR_UNSPECIFIED,
    BINARY_OPERATOR_ADD,
    BINARY_OPERATOR_SUBTRACT,
    BINARY_OPERATOR_MULTIPLY,
    BINARY_OPERATOR_DIVIDE,
    BINARY_OPERATOR_MODULO,
    BINARY_OPERATOR_EQUAL,
    BINARY_OPERATOR_NOT_EQUAL,
    BINARY_OPERATOR_LESS,
    BINARY_OPERATOR_LESS_EQUAL,
    BINARY_OPERATOR_GREATER,
    BINARY_OPERATOR_GREATER_EQUAL,
  ];

  static final $core.List<BinaryOperator?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 11);
  static BinaryOperator? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const BinaryOperator._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
