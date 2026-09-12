// This is a generated file - do not edit.
//
// Generated from storybundle/v1/compiled_story.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use variableTypeDescriptor instead')
const VariableType$json = {
  '1': 'VariableType',
  '2': [
    {'1': 'VARIABLE_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'VARIABLE_TYPE_INTEGER', '2': 1},
    {'1': 'VARIABLE_TYPE_STRING', '2': 2},
    {'1': 'VARIABLE_TYPE_BOOLEAN', '2': 3},
    {'1': 'VARIABLE_TYPE_DECIMAL', '2': 4},
    {'1': 'VARIABLE_TYPE_ARRAY_INTEGER', '2': 5},
    {'1': 'VARIABLE_TYPE_ARRAY_STRING', '2': 6},
    {'1': 'VARIABLE_TYPE_ARRAY_BOOLEAN', '2': 7},
    {'1': 'VARIABLE_TYPE_ARRAY_DECIMAL', '2': 8},
  ],
};

/// Descriptor for `VariableType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List variableTypeDescriptor = $convert.base64Decode(
    'CgxWYXJpYWJsZVR5cGUSHQoZVkFSSUFCTEVfVFlQRV9VTlNQRUNJRklFRBAAEhkKFVZBUklBQk'
    'xFX1RZUEVfSU5URUdFUhABEhgKFFZBUklBQkxFX1RZUEVfU1RSSU5HEAISGQoVVkFSSUFCTEVf'
    'VFlQRV9CT09MRUFOEAMSGQoVVkFSSUFCTEVfVFlQRV9ERUNJTUFMEAQSHwobVkFSSUFCTEVfVF'
    'lQRV9BUlJBWV9JTlRFR0VSEAUSHgoaVkFSSUFCTEVfVFlQRV9BUlJBWV9TVFJJTkcQBhIfChtW'
    'QVJJQUJMRV9UWVBFX0FSUkFZX0JPT0xFQU4QBxIfChtWQVJJQUJMRV9UWVBFX0FSUkFZX0RFQ0'
    'lNQUwQCA==');

@$core.Deprecated('Use assignmentOperatorDescriptor instead')
const AssignmentOperator$json = {
  '1': 'AssignmentOperator',
  '2': [
    {'1': 'ASSIGNMENT_OPERATOR_UNSPECIFIED', '2': 0},
    {'1': 'ASSIGNMENT_OPERATOR_SET', '2': 1},
    {'1': 'ASSIGNMENT_OPERATOR_ADD', '2': 2},
    {'1': 'ASSIGNMENT_OPERATOR_SUBTRACT', '2': 3},
  ],
};

/// Descriptor for `AssignmentOperator`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List assignmentOperatorDescriptor = $convert.base64Decode(
    'ChJBc3NpZ25tZW50T3BlcmF0b3ISIwofQVNTSUdOTUVOVF9PUEVSQVRPUl9VTlNQRUNJRklFRB'
    'AAEhsKF0FTU0lHTk1FTlRfT1BFUkFUT1JfU0VUEAESGwoXQVNTSUdOTUVOVF9PUEVSQVRPUl9B'
    'REQQAhIgChxBU1NJR05NRU5UX09QRVJBVE9SX1NVQlRSQUNUEAM=');

@$core.Deprecated('Use positionDescriptor instead')
const Position$json = {
  '1': 'Position',
  '2': [
    {'1': 'POSITION_UNSPECIFIED', '2': 0},
    {'1': 'POSITION_LEFT', '2': 1},
    {'1': 'POSITION_CENTER', '2': 2},
    {'1': 'POSITION_RIGHT', '2': 3},
  ],
};

/// Descriptor for `Position`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List positionDescriptor = $convert.base64Decode(
    'CghQb3NpdGlvbhIYChRQT1NJVElPTl9VTlNQRUNJRklFRBAAEhEKDVBPU0lUSU9OX0xFRlQQAR'
    'ITCg9QT1NJVElPTl9DRU5URVIQAhISCg5QT1NJVElPTl9SSUdIVBAD');

@$core.Deprecated('Use binaryOperatorDescriptor instead')
const BinaryOperator$json = {
  '1': 'BinaryOperator',
  '2': [
    {'1': 'BINARY_OPERATOR_UNSPECIFIED', '2': 0},
    {'1': 'BINARY_OPERATOR_ADD', '2': 1},
    {'1': 'BINARY_OPERATOR_SUBTRACT', '2': 2},
    {'1': 'BINARY_OPERATOR_MULTIPLY', '2': 3},
    {'1': 'BINARY_OPERATOR_DIVIDE', '2': 4},
    {'1': 'BINARY_OPERATOR_MODULO', '2': 5},
    {'1': 'BINARY_OPERATOR_EQUAL', '2': 6},
    {'1': 'BINARY_OPERATOR_NOT_EQUAL', '2': 7},
    {'1': 'BINARY_OPERATOR_LESS', '2': 8},
    {'1': 'BINARY_OPERATOR_LESS_EQUAL', '2': 9},
    {'1': 'BINARY_OPERATOR_GREATER', '2': 10},
    {'1': 'BINARY_OPERATOR_GREATER_EQUAL', '2': 11},
  ],
};

/// Descriptor for `BinaryOperator`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List binaryOperatorDescriptor = $convert.base64Decode(
    'Cg5CaW5hcnlPcGVyYXRvchIfChtCSU5BUllfT1BFUkFUT1JfVU5TUEVDSUZJRUQQABIXChNCSU'
    '5BUllfT1BFUkFUT1JfQUREEAESHAoYQklOQVJZX09QRVJBVE9SX1NVQlRSQUNUEAISHAoYQklO'
    'QVJZX09QRVJBVE9SX01VTFRJUExZEAMSGgoWQklOQVJZX09QRVJBVE9SX0RJVklERRAEEhoKFk'
    'JJTkFSWV9PUEVSQVRPUl9NT0RVTE8QBRIZChVCSU5BUllfT1BFUkFUT1JfRVFVQUwQBhIdChlC'
    'SU5BUllfT1BFUkFUT1JfTk9UX0VRVUFMEAcSGAoUQklOQVJZX09QRVJBVE9SX0xFU1MQCBIeCh'
    'pCSU5BUllfT1BFUkFUT1JfTEVTU19FUVVBTBAJEhsKF0JJTkFSWV9PUEVSQVRPUl9HUkVBVEVS'
    'EAoSIQodQklOQVJZX09QRVJBVE9SX0dSRUFURVJfRVFVQUwQCw==');

@$core.Deprecated('Use compiledStoryDescriptor instead')
const CompiledStory$json = {
  '1': 'CompiledStory',
  '2': [
    {'1': 'format_version', '3': 1, '4': 1, '5': 13, '10': 'formatVersion'},
    {
      '1': 'project',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ProjectMetadata',
      '10': 'project'
    },
    {
      '1': 'initialization',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Initialization',
      '10': 'initialization'
    },
    {
      '1': 'logic_blocks',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.LogicBlock',
      '10': 'logicBlocks'
    },
    {
      '1': 'scenes',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.Scene',
      '10': 'scenes'
    },
  ],
};

/// Descriptor for `CompiledStory`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List compiledStoryDescriptor = $convert.base64Decode(
    'Cg1Db21waWxlZFN0b3J5EiUKDmZvcm1hdF92ZXJzaW9uGAEgASgNUg1mb3JtYXRWZXJzaW9uEj'
    'kKB3Byb2plY3QYAiABKAsyHy5zdG9yeWJ1bmRsZS52MS5Qcm9qZWN0TWV0YWRhdGFSB3Byb2pl'
    'Y3QSRgoOaW5pdGlhbGl6YXRpb24YAyABKAsyHi5zdG9yeWJ1bmRsZS52MS5Jbml0aWFsaXphdG'
    'lvblIOaW5pdGlhbGl6YXRpb24SPQoMbG9naWNfYmxvY2tzGAQgAygLMhouc3RvcnlidW5kbGUu'
    'djEuTG9naWNCbG9ja1ILbG9naWNCbG9ja3MSLQoGc2NlbmVzGAUgAygLMhUuc3RvcnlidW5kbG'
    'UudjEuU2NlbmVSBnNjZW5lcw==');

@$core.Deprecated('Use projectMetadataDescriptor instead')
const ProjectMetadata$json = {
  '1': 'ProjectMetadata',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'version', '3': 3, '4': 1, '5': 9, '10': 'version'},
  ],
};

/// Descriptor for `ProjectMetadata`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List projectMetadataDescriptor = $convert.base64Decode(
    'Cg9Qcm9qZWN0TWV0YWRhdGESDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWUSGA'
    'oHdmVyc2lvbhgDIAEoCVIHdmVyc2lvbg==');

@$core.Deprecated('Use initializationDescriptor instead')
const Initialization$json = {
  '1': 'Initialization',
  '2': [
    {
      '1': 'variables',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.VariableDefinition',
      '10': 'variables'
    },
    {
      '1': 'actors',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.Actor',
      '10': 'actors'
    },
    {'1': 'start_scene', '3': 3, '4': 1, '5': 9, '10': 'startScene'},
  ],
};

/// Descriptor for `Initialization`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List initializationDescriptor = $convert.base64Decode(
    'Cg5Jbml0aWFsaXphdGlvbhJACgl2YXJpYWJsZXMYASADKAsyIi5zdG9yeWJ1bmRsZS52MS5WYX'
    'JpYWJsZURlZmluaXRpb25SCXZhcmlhYmxlcxItCgZhY3RvcnMYAiADKAsyFS5zdG9yeWJ1bmRs'
    'ZS52MS5BY3RvclIGYWN0b3JzEh8KC3N0YXJ0X3NjZW5lGAMgASgJUgpzdGFydFNjZW5l');

@$core.Deprecated('Use variableDefinitionDescriptor instead')
const VariableDefinition$json = {
  '1': 'VariableDefinition',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'type',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.storybundle.v1.VariableType',
      '10': 'type'
    },
    {
      '1': 'value',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'value'
    },
  ],
};

/// Descriptor for `VariableDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List variableDefinitionDescriptor = $convert.base64Decode(
    'ChJWYXJpYWJsZURlZmluaXRpb24SEgoEbmFtZRgBIAEoCVIEbmFtZRIwCgR0eXBlGAIgASgOMh'
    'wuc3RvcnlidW5kbGUudjEuVmFyaWFibGVUeXBlUgR0eXBlEjAKBXZhbHVlGAMgASgLMhouc3Rv'
    'cnlidW5kbGUudjEuRXhwcmVzc2lvblIFdmFsdWU=');

@$core.Deprecated('Use actorDescriptor instead')
const Actor$json = {
  '1': 'Actor',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'display_name',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'displayName'
    },
    {
      '1': 'portraits',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.Portrait',
      '10': 'portraits'
    },
  ],
};

/// Descriptor for `Actor`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List actorDescriptor = $convert.base64Decode(
    'CgVBY3RvchIOCgJpZBgBIAEoCVICaWQSRQoMZGlzcGxheV9uYW1lGAIgASgLMiIuc3RvcnlidW'
    '5kbGUudjEuSW50ZXJwb2xhdGVkU3RyaW5nUgtkaXNwbGF5TmFtZRI2Cglwb3J0cmFpdHMYAyAD'
    'KAsyGC5zdG9yeWJ1bmRsZS52MS5Qb3J0cmFpdFIJcG9ydHJhaXRz');

@$core.Deprecated('Use portraitDescriptor instead')
const Portrait$json = {
  '1': 'Portrait',
  '2': [
    {'1': 'emotion', '3': 1, '4': 1, '5': 9, '10': 'emotion'},
    {
      '1': 'asset_path',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'assetPath'
    },
  ],
};

/// Descriptor for `Portrait`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List portraitDescriptor = $convert.base64Decode(
    'CghQb3J0cmFpdBIYCgdlbW90aW9uGAEgASgJUgdlbW90aW9uEkEKCmFzc2V0X3BhdGgYAiABKA'
    'syIi5zdG9yeWJ1bmRsZS52MS5JbnRlcnBvbGF0ZWRTdHJpbmdSCWFzc2V0UGF0aA==');

@$core.Deprecated('Use interpolatedStringDescriptor instead')
const InterpolatedString$json = {
  '1': 'InterpolatedString',
  '2': [
    {
      '1': 'segments',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.StringSegment',
      '10': 'segments'
    },
  ],
};

/// Descriptor for `InterpolatedString`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List interpolatedStringDescriptor = $convert.base64Decode(
    'ChJJbnRlcnBvbGF0ZWRTdHJpbmcSOQoIc2VnbWVudHMYASADKAsyHS5zdG9yeWJ1bmRsZS52MS'
    '5TdHJpbmdTZWdtZW50UghzZWdtZW50cw==');

@$core.Deprecated('Use stringSegmentDescriptor instead')
const StringSegment$json = {
  '1': 'StringSegment',
  '2': [
    {'1': 'literal', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'literal'},
    {'1': 'variable', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'variable'},
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `StringSegment`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List stringSegmentDescriptor = $convert.base64Decode(
    'Cg1TdHJpbmdTZWdtZW50EhoKB2xpdGVyYWwYASABKAlIAFIHbGl0ZXJhbBIcCgh2YXJpYWJsZR'
    'gCIAEoCUgAUgh2YXJpYWJsZUIHCgV2YWx1ZQ==');

@$core.Deprecated('Use logicBlockDescriptor instead')
const LogicBlock$json = {
  '1': 'LogicBlock',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'parameters',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.LogicParameter',
      '10': 'parameters'
    },
    {
      '1': 'return_type',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.storybundle.v1.VariableType',
      '9': 0,
      '10': 'returnType',
      '17': true
    },
    {
      '1': 'body',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.PrepStatement',
      '10': 'body'
    },
  ],
  '8': [
    {'1': '_return_type'},
  ],
};

/// Descriptor for `LogicBlock`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List logicBlockDescriptor = $convert.base64Decode(
    'CgpMb2dpY0Jsb2NrEhIKBG5hbWUYASABKAlSBG5hbWUSPgoKcGFyYW1ldGVycxgCIAMoCzIeLn'
    'N0b3J5YnVuZGxlLnYxLkxvZ2ljUGFyYW1ldGVyUgpwYXJhbWV0ZXJzEkIKC3JldHVybl90eXBl'
    'GAMgASgOMhwuc3RvcnlidW5kbGUudjEuVmFyaWFibGVUeXBlSABSCnJldHVyblR5cGWIAQESMQ'
    'oEYm9keRgEIAMoCzIdLnN0b3J5YnVuZGxlLnYxLlByZXBTdGF0ZW1lbnRSBGJvZHlCDgoMX3Jl'
    'dHVybl90eXBl');

@$core.Deprecated('Use logicParameterDescriptor instead')
const LogicParameter$json = {
  '1': 'LogicParameter',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'type',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.storybundle.v1.VariableType',
      '10': 'type'
    },
  ],
};

/// Descriptor for `LogicParameter`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List logicParameterDescriptor = $convert.base64Decode(
    'Cg5Mb2dpY1BhcmFtZXRlchISCgRuYW1lGAEgASgJUgRuYW1lEjAKBHR5cGUYAiABKA4yHC5zdG'
    '9yeWJ1bmRsZS52MS5WYXJpYWJsZVR5cGVSBHR5cGU=');

@$core.Deprecated('Use sceneDescriptor instead')
const Scene$json = {
  '1': 'Scene',
  '2': [
    {'1': 'label', '3': 1, '4': 1, '5': 9, '10': 'label'},
    {
      '1': 'prep',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PrepBlock',
      '9': 0,
      '10': 'prep',
      '17': true
    },
    {
      '1': 'story',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.StoryBlock',
      '10': 'story'
    },
  ],
  '8': [
    {'1': '_prep'},
  ],
};

/// Descriptor for `Scene`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sceneDescriptor = $convert.base64Decode(
    'CgVTY2VuZRIUCgVsYWJlbBgBIAEoCVIFbGFiZWwSMgoEcHJlcBgCIAEoCzIZLnN0b3J5YnVuZG'
    'xlLnYxLlByZXBCbG9ja0gAUgRwcmVwiAEBEjAKBXN0b3J5GAMgASgLMhouc3RvcnlidW5kbGUu'
    'djEuU3RvcnlCbG9ja1IFc3RvcnlCBwoFX3ByZXA=');

@$core.Deprecated('Use prepBlockDescriptor instead')
const PrepBlock$json = {
  '1': 'PrepBlock',
  '2': [
    {
      '1': 'statements',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.PrepStatement',
      '10': 'statements'
    },
  ],
};

/// Descriptor for `PrepBlock`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List prepBlockDescriptor = $convert.base64Decode(
    'CglQcmVwQmxvY2sSPQoKc3RhdGVtZW50cxgBIAMoCzIdLnN0b3J5YnVuZGxlLnYxLlByZXBTdG'
    'F0ZW1lbnRSCnN0YXRlbWVudHM=');

@$core.Deprecated('Use storyBlockDescriptor instead')
const StoryBlock$json = {
  '1': 'StoryBlock',
  '2': [
    {
      '1': 'statements',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.StoryStatement',
      '10': 'statements'
    },
  ],
};

/// Descriptor for `StoryBlock`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storyBlockDescriptor = $convert.base64Decode(
    'CgpTdG9yeUJsb2NrEj4KCnN0YXRlbWVudHMYASADKAsyHi5zdG9yeWJ1bmRsZS52MS5TdG9yeV'
    'N0YXRlbWVudFIKc3RhdGVtZW50cw==');

@$core.Deprecated('Use prepStatementDescriptor instead')
const PrepStatement$json = {
  '1': 'PrepStatement',
  '2': [
    {
      '1': 'background',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.BackgroundDirective',
      '9': 0,
      '10': 'background'
    },
    {
      '1': 'bgm',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.BgmDirective',
      '9': 0,
      '10': 'bgm'
    },
    {
      '1': 'sfx',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.SfxDirective',
      '9': 0,
      '10': 'sfx'
    },
    {
      '1': 'variable_definition',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.VariableDefinition',
      '9': 0,
      '10': 'variableDefinition'
    },
    {
      '1': 'variable_assignment',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.VariableAssignment',
      '9': 0,
      '10': 'variableAssignment'
    },
    {
      '1': 'call',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.CallExpression',
      '9': 0,
      '10': 'call'
    },
    {
      '1': 'if_else',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PrepIfElse',
      '9': 0,
      '10': 'ifElse'
    },
    {
      '1': 'for_snapshot',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PrepForSnapshot',
      '9': 0,
      '10': 'forSnapshot'
    },
    {
      '1': 'repeat',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PrepRepeat',
      '9': 0,
      '10': 'repeat'
    },
    {
      '1': 'break_loop',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'breakLoop'
    },
    {
      '1': 'continue_loop',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'continueLoop'
    },
    {
      '1': 'return_statement',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ReturnStatement',
      '9': 0,
      '10': 'returnStatement'
    },
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `PrepStatement`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List prepStatementDescriptor = $convert.base64Decode(
    'Cg1QcmVwU3RhdGVtZW50EkUKCmJhY2tncm91bmQYASABKAsyIy5zdG9yeWJ1bmRsZS52MS5CYW'
    'NrZ3JvdW5kRGlyZWN0aXZlSABSCmJhY2tncm91bmQSMAoDYmdtGAIgASgLMhwuc3RvcnlidW5k'
    'bGUudjEuQmdtRGlyZWN0aXZlSABSA2JnbRIwCgNzZngYAyABKAsyHC5zdG9yeWJ1bmRsZS52MS'
    '5TZnhEaXJlY3RpdmVIAFIDc2Z4ElUKE3ZhcmlhYmxlX2RlZmluaXRpb24YBCABKAsyIi5zdG9y'
    'eWJ1bmRsZS52MS5WYXJpYWJsZURlZmluaXRpb25IAFISdmFyaWFibGVEZWZpbml0aW9uElUKE3'
    'ZhcmlhYmxlX2Fzc2lnbm1lbnQYBSABKAsyIi5zdG9yeWJ1bmRsZS52MS5WYXJpYWJsZUFzc2ln'
    'bm1lbnRIAFISdmFyaWFibGVBc3NpZ25tZW50EjQKBGNhbGwYBiABKAsyHi5zdG9yeWJ1bmRsZS'
    '52MS5DYWxsRXhwcmVzc2lvbkgAUgRjYWxsEjUKB2lmX2Vsc2UYByABKAsyGi5zdG9yeWJ1bmRs'
    'ZS52MS5QcmVwSWZFbHNlSABSBmlmRWxzZRJECgxmb3Jfc25hcHNob3QYCCABKAsyHy5zdG9yeW'
    'J1bmRsZS52MS5QcmVwRm9yU25hcHNob3RIAFILZm9yU25hcHNob3QSNAoGcmVwZWF0GAkgASgL'
    'Mhouc3RvcnlidW5kbGUudjEuUHJlcFJlcGVhdEgAUgZyZXBlYXQSNwoKYnJlYWtfbG9vcBgKIA'
    'EoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eUgAUglicmVha0xvb3ASPQoNY29udGludWVfbG9v'
    'cBgLIAEoCzIWLmdvb2dsZS5wcm90b2J1Zi5FbXB0eUgAUgxjb250aW51ZUxvb3ASTAoQcmV0dX'
    'JuX3N0YXRlbWVudBgMIAEoCzIfLnN0b3J5YnVuZGxlLnYxLlJldHVyblN0YXRlbWVudEgAUg9y'
    'ZXR1cm5TdGF0ZW1lbnRCBwoFdmFsdWU=');

@$core.Deprecated('Use backgroundDirectiveDescriptor instead')
const BackgroundDirective$json = {
  '1': 'BackgroundDirective',
  '2': [
    {
      '1': 'asset_path',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'assetPath'
    },
  ],
};

/// Descriptor for `BackgroundDirective`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List backgroundDirectiveDescriptor = $convert.base64Decode(
    'ChNCYWNrZ3JvdW5kRGlyZWN0aXZlEkEKCmFzc2V0X3BhdGgYASABKAsyIi5zdG9yeWJ1bmRsZS'
    '52MS5JbnRlcnBvbGF0ZWRTdHJpbmdSCWFzc2V0UGF0aA==');

@$core.Deprecated('Use bgmDirectiveDescriptor instead')
const BgmDirective$json = {
  '1': 'BgmDirective',
  '2': [
    {
      '1': 'asset_path',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '9': 0,
      '10': 'assetPath'
    },
    {
      '1': 'stop',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'stop'
    },
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `BgmDirective`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bgmDirectiveDescriptor = $convert.base64Decode(
    'CgxCZ21EaXJlY3RpdmUSQwoKYXNzZXRfcGF0aBgBIAEoCzIiLnN0b3J5YnVuZGxlLnYxLkludG'
    'VycG9sYXRlZFN0cmluZ0gAUglhc3NldFBhdGgSLAoEc3RvcBgCIAEoCzIWLmdvb2dsZS5wcm90'
    'b2J1Zi5FbXB0eUgAUgRzdG9wQgcKBXZhbHVl');

@$core.Deprecated('Use sfxDirectiveDescriptor instead')
const SfxDirective$json = {
  '1': 'SfxDirective',
  '2': [
    {
      '1': 'asset_path',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'assetPath'
    },
  ],
};

/// Descriptor for `SfxDirective`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sfxDirectiveDescriptor = $convert.base64Decode(
    'CgxTZnhEaXJlY3RpdmUSQQoKYXNzZXRfcGF0aBgBIAEoCzIiLnN0b3J5YnVuZGxlLnYxLkludG'
    'VycG9sYXRlZFN0cmluZ1IJYXNzZXRQYXRo');

@$core.Deprecated('Use variableAssignmentDescriptor instead')
const VariableAssignment$json = {
  '1': 'VariableAssignment',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'operator',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.storybundle.v1.AssignmentOperator',
      '10': 'operator'
    },
    {
      '1': 'value',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'value'
    },
  ],
};

/// Descriptor for `VariableAssignment`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List variableAssignmentDescriptor = $convert.base64Decode(
    'ChJWYXJpYWJsZUFzc2lnbm1lbnQSEgoEbmFtZRgBIAEoCVIEbmFtZRI+CghvcGVyYXRvchgCIA'
    'EoDjIiLnN0b3J5YnVuZGxlLnYxLkFzc2lnbm1lbnRPcGVyYXRvclIIb3BlcmF0b3ISMAoFdmFs'
    'dWUYAyABKAsyGi5zdG9yeWJ1bmRsZS52MS5FeHByZXNzaW9uUgV2YWx1ZQ==');

@$core.Deprecated('Use prepStatementListDescriptor instead')
const PrepStatementList$json = {
  '1': 'PrepStatementList',
  '2': [
    {
      '1': 'statements',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.PrepStatement',
      '10': 'statements'
    },
  ],
};

/// Descriptor for `PrepStatementList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List prepStatementListDescriptor = $convert.base64Decode(
    'ChFQcmVwU3RhdGVtZW50TGlzdBI9CgpzdGF0ZW1lbnRzGAEgAygLMh0uc3RvcnlidW5kbGUudj'
    'EuUHJlcFN0YXRlbWVudFIKc3RhdGVtZW50cw==');

@$core.Deprecated('Use prepIfElseDescriptor instead')
const PrepIfElse$json = {
  '1': 'PrepIfElse',
  '2': [
    {
      '1': 'condition',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'condition'
    },
    {
      '1': 'then_branch',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PrepStatementList',
      '10': 'thenBranch'
    },
    {
      '1': 'else_branch',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PrepStatementList',
      '9': 0,
      '10': 'elseBranch',
      '17': true
    },
  ],
  '8': [
    {'1': '_else_branch'},
  ],
};

/// Descriptor for `PrepIfElse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List prepIfElseDescriptor = $convert.base64Decode(
    'CgpQcmVwSWZFbHNlEjgKCWNvbmRpdGlvbhgBIAEoCzIaLnN0b3J5YnVuZGxlLnYxLkV4cHJlc3'
    'Npb25SCWNvbmRpdGlvbhJCCgt0aGVuX2JyYW5jaBgCIAEoCzIhLnN0b3J5YnVuZGxlLnYxLlBy'
    'ZXBTdGF0ZW1lbnRMaXN0Ugp0aGVuQnJhbmNoEkcKC2Vsc2VfYnJhbmNoGAMgASgLMiEuc3Rvcn'
    'lidW5kbGUudjEuUHJlcFN0YXRlbWVudExpc3RIAFIKZWxzZUJyYW5jaIgBAUIOCgxfZWxzZV9i'
    'cmFuY2g=');

@$core.Deprecated('Use prepForSnapshotDescriptor instead')
const PrepForSnapshot$json = {
  '1': 'PrepForSnapshot',
  '2': [
    {'1': 'item_name', '3': 1, '4': 1, '5': 9, '10': 'itemName'},
    {'1': 'array_name', '3': 2, '4': 1, '5': 9, '10': 'arrayName'},
    {
      '1': 'body',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.PrepStatement',
      '10': 'body'
    },
  ],
};

/// Descriptor for `PrepForSnapshot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List prepForSnapshotDescriptor = $convert.base64Decode(
    'Cg9QcmVwRm9yU25hcHNob3QSGwoJaXRlbV9uYW1lGAEgASgJUghpdGVtTmFtZRIdCgphcnJheV'
    '9uYW1lGAIgASgJUglhcnJheU5hbWUSMQoEYm9keRgDIAMoCzIdLnN0b3J5YnVuZGxlLnYxLlBy'
    'ZXBTdGF0ZW1lbnRSBGJvZHk=');

@$core.Deprecated('Use prepRepeatDescriptor instead')
const PrepRepeat$json = {
  '1': 'PrepRepeat',
  '2': [
    {
      '1': 'count',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.RepeatCount',
      '10': 'count'
    },
    {
      '1': 'body',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.PrepStatement',
      '10': 'body'
    },
  ],
};

/// Descriptor for `PrepRepeat`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List prepRepeatDescriptor = $convert.base64Decode(
    'CgpQcmVwUmVwZWF0EjEKBWNvdW50GAEgASgLMhsuc3RvcnlidW5kbGUudjEuUmVwZWF0Q291bn'
    'RSBWNvdW50EjEKBGJvZHkYAiADKAsyHS5zdG9yeWJ1bmRsZS52MS5QcmVwU3RhdGVtZW50UgRi'
    'b2R5');

@$core.Deprecated('Use returnStatementDescriptor instead')
const ReturnStatement$json = {
  '1': 'ReturnStatement',
  '2': [
    {
      '1': 'value',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '9': 0,
      '10': 'value',
      '17': true
    },
  ],
  '8': [
    {'1': '_value'},
  ],
};

/// Descriptor for `ReturnStatement`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List returnStatementDescriptor = $convert.base64Decode(
    'Cg9SZXR1cm5TdGF0ZW1lbnQSNQoFdmFsdWUYASABKAsyGi5zdG9yeWJ1bmRsZS52MS5FeHByZX'
    'NzaW9uSABSBXZhbHVliAEBQggKBl92YWx1ZQ==');

@$core.Deprecated('Use storyStatementDescriptor instead')
const StoryStatement$json = {
  '1': 'StoryStatement',
  '2': [
    {
      '1': 'narration',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Narration',
      '9': 0,
      '10': 'narration'
    },
    {
      '1': 'variable_output',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.VariableOutput',
      '9': 0,
      '10': 'variableOutput'
    },
    {
      '1': 'dialogue',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Dialogue',
      '9': 0,
      '10': 'dialogue'
    },
    {
      '1': 'if_else',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.StoryIfElse',
      '9': 0,
      '10': 'ifElse'
    },
    {
      '1': 'choice',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceBlock',
      '9': 0,
      '10': 'choice'
    },
    {
      '1': 'jump',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Jump',
      '9': 0,
      '10': 'jump'
    },
    {
      '1': 'end',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'end'
    },
    {
      '1': 'sfx',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.SfxDirective',
      '9': 0,
      '10': 'sfx'
    },
    {
      '1': 'for_snapshot',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.StoryForSnapshot',
      '9': 0,
      '10': 'forSnapshot'
    },
    {
      '1': 'repeat',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.StoryRepeat',
      '9': 0,
      '10': 'repeat'
    },
    {
      '1': 'break_loop',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'breakLoop'
    },
    {
      '1': 'continue_loop',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'continueLoop'
    },
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `StoryStatement`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storyStatementDescriptor = $convert.base64Decode(
    'Cg5TdG9yeVN0YXRlbWVudBI5CgluYXJyYXRpb24YASABKAsyGS5zdG9yeWJ1bmRsZS52MS5OYX'
    'JyYXRpb25IAFIJbmFycmF0aW9uEkkKD3ZhcmlhYmxlX291dHB1dBgCIAEoCzIeLnN0b3J5YnVu'
    'ZGxlLnYxLlZhcmlhYmxlT3V0cHV0SABSDnZhcmlhYmxlT3V0cHV0EjYKCGRpYWxvZ3VlGAMgAS'
    'gLMhguc3RvcnlidW5kbGUudjEuRGlhbG9ndWVIAFIIZGlhbG9ndWUSNgoHaWZfZWxzZRgEIAEo'
    'CzIbLnN0b3J5YnVuZGxlLnYxLlN0b3J5SWZFbHNlSABSBmlmRWxzZRI1CgZjaG9pY2UYBSABKA'
    'syGy5zdG9yeWJ1bmRsZS52MS5DaG9pY2VCbG9ja0gAUgZjaG9pY2USKgoEanVtcBgGIAEoCzIU'
    'LnN0b3J5YnVuZGxlLnYxLkp1bXBIAFIEanVtcBIqCgNlbmQYByABKAsyFi5nb29nbGUucHJvdG'
    '9idWYuRW1wdHlIAFIDZW5kEjAKA3NmeBgIIAEoCzIcLnN0b3J5YnVuZGxlLnYxLlNmeERpcmVj'
    'dGl2ZUgAUgNzZngSRQoMZm9yX3NuYXBzaG90GAkgASgLMiAuc3RvcnlidW5kbGUudjEuU3Rvcn'
    'lGb3JTbmFwc2hvdEgAUgtmb3JTbmFwc2hvdBI1CgZyZXBlYXQYCiABKAsyGy5zdG9yeWJ1bmRs'
    'ZS52MS5TdG9yeVJlcGVhdEgAUgZyZXBlYXQSNwoKYnJlYWtfbG9vcBgLIAEoCzIWLmdvb2dsZS'
    '5wcm90b2J1Zi5FbXB0eUgAUglicmVha0xvb3ASPQoNY29udGludWVfbG9vcBgMIAEoCzIWLmdv'
    'b2dsZS5wcm90b2J1Zi5FbXB0eUgAUgxjb250aW51ZUxvb3BCBwoFdmFsdWU=');

@$core.Deprecated('Use narrationDescriptor instead')
const Narration$json = {
  '1': 'Narration',
  '2': [
    {
      '1': 'text',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'text'
    },
  ],
};

/// Descriptor for `Narration`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List narrationDescriptor = $convert.base64Decode(
    'CglOYXJyYXRpb24SNgoEdGV4dBgBIAEoCzIiLnN0b3J5YnVuZGxlLnYxLkludGVycG9sYXRlZF'
    'N0cmluZ1IEdGV4dA==');

@$core.Deprecated('Use variableOutputDescriptor instead')
const VariableOutput$json = {
  '1': 'VariableOutput',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `VariableOutput`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List variableOutputDescriptor =
    $convert.base64Decode('Cg5WYXJpYWJsZU91dHB1dBISCgRuYW1lGAEgASgJUgRuYW1l');

@$core.Deprecated('Use dialogueDescriptor instead')
const Dialogue$json = {
  '1': 'Dialogue',
  '2': [
    {'1': 'actor_id', '3': 1, '4': 1, '5': 9, '10': 'actorId'},
    {
      '1': 'name_only',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Empty',
      '9': 0,
      '10': 'nameOnly'
    },
    {
      '1': 'portrait',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.PortraitDialogue',
      '9': 0,
      '10': 'portrait'
    },
    {
      '1': 'text',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'text'
    },
  ],
  '8': [
    {'1': 'form'},
  ],
};

/// Descriptor for `Dialogue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List dialogueDescriptor = $convert.base64Decode(
    'CghEaWFsb2d1ZRIZCghhY3Rvcl9pZBgBIAEoCVIHYWN0b3JJZBI1CgluYW1lX29ubHkYAiABKA'
    'syFi5nb29nbGUucHJvdG9idWYuRW1wdHlIAFIIbmFtZU9ubHkSPgoIcG9ydHJhaXQYAyABKAsy'
    'IC5zdG9yeWJ1bmRsZS52MS5Qb3J0cmFpdERpYWxvZ3VlSABSCHBvcnRyYWl0EjYKBHRleHQYBC'
    'ABKAsyIi5zdG9yeWJ1bmRsZS52MS5JbnRlcnBvbGF0ZWRTdHJpbmdSBHRleHRCBgoEZm9ybQ==');

@$core.Deprecated('Use portraitDialogueDescriptor instead')
const PortraitDialogue$json = {
  '1': 'PortraitDialogue',
  '2': [
    {'1': 'emotion', '3': 1, '4': 1, '5': 9, '10': 'emotion'},
    {
      '1': 'position',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.storybundle.v1.Position',
      '10': 'position'
    },
  ],
};

/// Descriptor for `PortraitDialogue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List portraitDialogueDescriptor = $convert.base64Decode(
    'ChBQb3J0cmFpdERpYWxvZ3VlEhgKB2Vtb3Rpb24YASABKAlSB2Vtb3Rpb24SNAoIcG9zaXRpb2'
    '4YAiABKA4yGC5zdG9yeWJ1bmRsZS52MS5Qb3NpdGlvblIIcG9zaXRpb24=');

@$core.Deprecated('Use storyStatementListDescriptor instead')
const StoryStatementList$json = {
  '1': 'StoryStatementList',
  '2': [
    {
      '1': 'statements',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.StoryStatement',
      '10': 'statements'
    },
  ],
};

/// Descriptor for `StoryStatementList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storyStatementListDescriptor = $convert.base64Decode(
    'ChJTdG9yeVN0YXRlbWVudExpc3QSPgoKc3RhdGVtZW50cxgBIAMoCzIeLnN0b3J5YnVuZGxlLn'
    'YxLlN0b3J5U3RhdGVtZW50UgpzdGF0ZW1lbnRz');

@$core.Deprecated('Use storyIfElseDescriptor instead')
const StoryIfElse$json = {
  '1': 'StoryIfElse',
  '2': [
    {
      '1': 'condition',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'condition'
    },
    {
      '1': 'then_branch',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.StoryStatementList',
      '10': 'thenBranch'
    },
    {
      '1': 'else_branch',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.StoryStatementList',
      '9': 0,
      '10': 'elseBranch',
      '17': true
    },
  ],
  '8': [
    {'1': '_else_branch'},
  ],
};

/// Descriptor for `StoryIfElse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storyIfElseDescriptor = $convert.base64Decode(
    'CgtTdG9yeUlmRWxzZRI4Cgljb25kaXRpb24YASABKAsyGi5zdG9yeWJ1bmRsZS52MS5FeHByZX'
    'NzaW9uUgljb25kaXRpb24SQwoLdGhlbl9icmFuY2gYAiABKAsyIi5zdG9yeWJ1bmRsZS52MS5T'
    'dG9yeVN0YXRlbWVudExpc3RSCnRoZW5CcmFuY2gSSAoLZWxzZV9icmFuY2gYAyABKAsyIi5zdG'
    '9yeWJ1bmRsZS52MS5TdG9yeVN0YXRlbWVudExpc3RIAFIKZWxzZUJyYW5jaIgBAUIOCgxfZWxz'
    'ZV9icmFuY2g=');

@$core.Deprecated('Use choiceBlockDescriptor instead')
const ChoiceBlock$json = {
  '1': 'ChoiceBlock',
  '2': [
    {
      '1': 'entries',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.ChoiceEntry',
      '10': 'entries'
    },
  ],
};

/// Descriptor for `ChoiceBlock`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceBlockDescriptor = $convert.base64Decode(
    'CgtDaG9pY2VCbG9jaxI1CgdlbnRyaWVzGAEgAygLMhsuc3RvcnlidW5kbGUudjEuQ2hvaWNlRW'
    '50cnlSB2VudHJpZXM=');

@$core.Deprecated('Use choiceEntryDescriptor instead')
const ChoiceEntry$json = {
  '1': 'ChoiceEntry',
  '2': [
    {
      '1': 'option',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceOption',
      '9': 0,
      '10': 'option'
    },
    {
      '1': 'if_entry',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceIf',
      '9': 0,
      '10': 'ifEntry'
    },
    {
      '1': 'repeat',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceRepeat',
      '9': 0,
      '10': 'repeat'
    },
    {
      '1': 'for_snapshot',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceForSnapshot',
      '9': 0,
      '10': 'forSnapshot'
    },
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `ChoiceEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceEntryDescriptor = $convert.base64Decode(
    'CgtDaG9pY2VFbnRyeRI2CgZvcHRpb24YASABKAsyHC5zdG9yeWJ1bmRsZS52MS5DaG9pY2VPcH'
    'Rpb25IAFIGb3B0aW9uEjUKCGlmX2VudHJ5GAIgASgLMhguc3RvcnlidW5kbGUudjEuQ2hvaWNl'
    'SWZIAFIHaWZFbnRyeRI2CgZyZXBlYXQYAyABKAsyHC5zdG9yeWJ1bmRsZS52MS5DaG9pY2VSZX'
    'BlYXRIAFIGcmVwZWF0EkYKDGZvcl9zbmFwc2hvdBgEIAEoCzIhLnN0b3J5YnVuZGxlLnYxLkNo'
    'b2ljZUZvclNuYXBzaG90SABSC2ZvclNuYXBzaG90QgcKBXZhbHVl');

@$core.Deprecated('Use choiceOptionDescriptor instead')
const ChoiceOption$json = {
  '1': 'ChoiceOption',
  '2': [
    {
      '1': 'text',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '10': 'text'
    },
    {'1': 'target', '3': 2, '4': 1, '5': 9, '10': 'target'},
  ],
};

/// Descriptor for `ChoiceOption`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceOptionDescriptor = $convert.base64Decode(
    'CgxDaG9pY2VPcHRpb24SNgoEdGV4dBgBIAEoCzIiLnN0b3J5YnVuZGxlLnYxLkludGVycG9sYX'
    'RlZFN0cmluZ1IEdGV4dBIWCgZ0YXJnZXQYAiABKAlSBnRhcmdldA==');

@$core.Deprecated('Use choiceEntryListDescriptor instead')
const ChoiceEntryList$json = {
  '1': 'ChoiceEntryList',
  '2': [
    {
      '1': 'entries',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.ChoiceEntry',
      '10': 'entries'
    },
  ],
};

/// Descriptor for `ChoiceEntryList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceEntryListDescriptor = $convert.base64Decode(
    'Cg9DaG9pY2VFbnRyeUxpc3QSNQoHZW50cmllcxgBIAMoCzIbLnN0b3J5YnVuZGxlLnYxLkNob2'
    'ljZUVudHJ5UgdlbnRyaWVz');

@$core.Deprecated('Use choiceIfDescriptor instead')
const ChoiceIf$json = {
  '1': 'ChoiceIf',
  '2': [
    {
      '1': 'condition',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'condition'
    },
    {
      '1': 'body',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceEntryList',
      '10': 'body'
    },
  ],
};

/// Descriptor for `ChoiceIf`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceIfDescriptor = $convert.base64Decode(
    'CghDaG9pY2VJZhI4Cgljb25kaXRpb24YASABKAsyGi5zdG9yeWJ1bmRsZS52MS5FeHByZXNzaW'
    '9uUgljb25kaXRpb24SMwoEYm9keRgCIAEoCzIfLnN0b3J5YnVuZGxlLnYxLkNob2ljZUVudHJ5'
    'TGlzdFIEYm9keQ==');

@$core.Deprecated('Use choiceRepeatDescriptor instead')
const ChoiceRepeat$json = {
  '1': 'ChoiceRepeat',
  '2': [
    {
      '1': 'count',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.RepeatCount',
      '10': 'count'
    },
    {
      '1': 'body',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceEntryList',
      '10': 'body'
    },
  ],
};

/// Descriptor for `ChoiceRepeat`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceRepeatDescriptor = $convert.base64Decode(
    'CgxDaG9pY2VSZXBlYXQSMQoFY291bnQYASABKAsyGy5zdG9yeWJ1bmRsZS52MS5SZXBlYXRDb3'
    'VudFIFY291bnQSMwoEYm9keRgCIAEoCzIfLnN0b3J5YnVuZGxlLnYxLkNob2ljZUVudHJ5TGlz'
    'dFIEYm9keQ==');

@$core.Deprecated('Use choiceForSnapshotDescriptor instead')
const ChoiceForSnapshot$json = {
  '1': 'ChoiceForSnapshot',
  '2': [
    {'1': 'item_name', '3': 1, '4': 1, '5': 9, '10': 'itemName'},
    {'1': 'array_name', '3': 2, '4': 1, '5': 9, '10': 'arrayName'},
    {
      '1': 'body',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ChoiceEntryList',
      '10': 'body'
    },
  ],
};

/// Descriptor for `ChoiceForSnapshot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List choiceForSnapshotDescriptor = $convert.base64Decode(
    'ChFDaG9pY2VGb3JTbmFwc2hvdBIbCglpdGVtX25hbWUYASABKAlSCGl0ZW1OYW1lEh0KCmFycm'
    'F5X25hbWUYAiABKAlSCWFycmF5TmFtZRIzCgRib2R5GAMgASgLMh8uc3RvcnlidW5kbGUudjEu'
    'Q2hvaWNlRW50cnlMaXN0UgRib2R5');

@$core.Deprecated('Use jumpDescriptor instead')
const Jump$json = {
  '1': 'Jump',
  '2': [
    {'1': 'target', '3': 1, '4': 1, '5': 9, '10': 'target'},
  ],
};

/// Descriptor for `Jump`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List jumpDescriptor =
    $convert.base64Decode('CgRKdW1wEhYKBnRhcmdldBgBIAEoCVIGdGFyZ2V0');

@$core.Deprecated('Use storyForSnapshotDescriptor instead')
const StoryForSnapshot$json = {
  '1': 'StoryForSnapshot',
  '2': [
    {'1': 'item_name', '3': 1, '4': 1, '5': 9, '10': 'itemName'},
    {'1': 'array_name', '3': 2, '4': 1, '5': 9, '10': 'arrayName'},
    {
      '1': 'body',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.StoryStatement',
      '10': 'body'
    },
  ],
};

/// Descriptor for `StoryForSnapshot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storyForSnapshotDescriptor = $convert.base64Decode(
    'ChBTdG9yeUZvclNuYXBzaG90EhsKCWl0ZW1fbmFtZRgBIAEoCVIIaXRlbU5hbWUSHQoKYXJyYX'
    'lfbmFtZRgCIAEoCVIJYXJyYXlOYW1lEjIKBGJvZHkYAyADKAsyHi5zdG9yeWJ1bmRsZS52MS5T'
    'dG9yeVN0YXRlbWVudFIEYm9keQ==');

@$core.Deprecated('Use storyRepeatDescriptor instead')
const StoryRepeat$json = {
  '1': 'StoryRepeat',
  '2': [
    {
      '1': 'count',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.RepeatCount',
      '10': 'count'
    },
    {
      '1': 'body',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.StoryStatement',
      '10': 'body'
    },
  ],
};

/// Descriptor for `StoryRepeat`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storyRepeatDescriptor = $convert.base64Decode(
    'CgtTdG9yeVJlcGVhdBIxCgVjb3VudBgBIAEoCzIbLnN0b3J5YnVuZGxlLnYxLlJlcGVhdENvdW'
    '50UgVjb3VudBIyCgRib2R5GAIgAygLMh4uc3RvcnlidW5kbGUudjEuU3RvcnlTdGF0ZW1lbnRS'
    'BGJvZHk=');

@$core.Deprecated('Use repeatCountDescriptor instead')
const RepeatCount$json = {
  '1': 'RepeatCount',
  '2': [
    {'1': 'integer', '3': 1, '4': 1, '5': 3, '9': 0, '10': 'integer'},
    {'1': 'variable', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'variable'},
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `RepeatCount`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List repeatCountDescriptor = $convert.base64Decode(
    'CgtSZXBlYXRDb3VudBIaCgdpbnRlZ2VyGAEgASgDSABSB2ludGVnZXISHAoIdmFyaWFibGUYAi'
    'ABKAlIAFIIdmFyaWFibGVCBwoFdmFsdWU=');

@$core.Deprecated('Use expressionDescriptor instead')
const Expression$json = {
  '1': 'Expression',
  '2': [
    {'1': 'integer', '3': 1, '4': 1, '5': 3, '9': 0, '10': 'integer'},
    {
      '1': 'decimal',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.DecimalValue',
      '9': 0,
      '10': 'decimal'
    },
    {'1': 'boolean', '3': 3, '4': 1, '5': 8, '9': 0, '10': 'boolean'},
    {
      '1': 'string_value',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.InterpolatedString',
      '9': 0,
      '10': 'stringValue'
    },
    {
      '1': 'variable',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.VariableReference',
      '9': 0,
      '10': 'variable'
    },
    {
      '1': 'binary',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.BinaryExpression',
      '9': 0,
      '10': 'binary'
    },
    {
      '1': 'call',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.CallExpression',
      '9': 0,
      '10': 'call'
    },
    {
      '1': 'list',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.ListExpression',
      '9': 0,
      '10': 'list'
    },
  ],
  '8': [
    {'1': 'value'},
  ],
};

/// Descriptor for `Expression`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List expressionDescriptor = $convert.base64Decode(
    'CgpFeHByZXNzaW9uEhoKB2ludGVnZXIYASABKANIAFIHaW50ZWdlchI4CgdkZWNpbWFsGAIgAS'
    'gLMhwuc3RvcnlidW5kbGUudjEuRGVjaW1hbFZhbHVlSABSB2RlY2ltYWwSGgoHYm9vbGVhbhgD'
    'IAEoCEgAUgdib29sZWFuEkcKDHN0cmluZ192YWx1ZRgEIAEoCzIiLnN0b3J5YnVuZGxlLnYxLk'
    'ludGVycG9sYXRlZFN0cmluZ0gAUgtzdHJpbmdWYWx1ZRI/Cgh2YXJpYWJsZRgFIAEoCzIhLnN0'
    'b3J5YnVuZGxlLnYxLlZhcmlhYmxlUmVmZXJlbmNlSABSCHZhcmlhYmxlEjoKBmJpbmFyeRgGIA'
    'EoCzIgLnN0b3J5YnVuZGxlLnYxLkJpbmFyeUV4cHJlc3Npb25IAFIGYmluYXJ5EjQKBGNhbGwY'
    'ByABKAsyHi5zdG9yeWJ1bmRsZS52MS5DYWxsRXhwcmVzc2lvbkgAUgRjYWxsEjQKBGxpc3QYCC'
    'ABKAsyHi5zdG9yeWJ1bmRsZS52MS5MaXN0RXhwcmVzc2lvbkgAUgRsaXN0QgcKBXZhbHVl');

@$core.Deprecated('Use decimalValueDescriptor instead')
const DecimalValue$json = {
  '1': 'DecimalValue',
  '2': [
    {'1': 'canonical', '3': 1, '4': 1, '5': 9, '10': 'canonical'},
  ],
};

/// Descriptor for `DecimalValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List decimalValueDescriptor = $convert.base64Decode(
    'CgxEZWNpbWFsVmFsdWUSHAoJY2Fub25pY2FsGAEgASgJUgljYW5vbmljYWw=');

@$core.Deprecated('Use variableReferenceDescriptor instead')
const VariableReference$json = {
  '1': 'VariableReference',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `VariableReference`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List variableReferenceDescriptor = $convert
    .base64Decode('ChFWYXJpYWJsZVJlZmVyZW5jZRISCgRuYW1lGAEgASgJUgRuYW1l');

@$core.Deprecated('Use binaryExpressionDescriptor instead')
const BinaryExpression$json = {
  '1': 'BinaryExpression',
  '2': [
    {
      '1': 'left',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'left'
    },
    {
      '1': 'operator',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.storybundle.v1.BinaryOperator',
      '10': 'operator'
    },
    {
      '1': 'right',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'right'
    },
  ],
};

/// Descriptor for `BinaryExpression`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List binaryExpressionDescriptor = $convert.base64Decode(
    'ChBCaW5hcnlFeHByZXNzaW9uEi4KBGxlZnQYASABKAsyGi5zdG9yeWJ1bmRsZS52MS5FeHByZX'
    'NzaW9uUgRsZWZ0EjoKCG9wZXJhdG9yGAIgASgOMh4uc3RvcnlidW5kbGUudjEuQmluYXJ5T3Bl'
    'cmF0b3JSCG9wZXJhdG9yEjAKBXJpZ2h0GAMgASgLMhouc3RvcnlidW5kbGUudjEuRXhwcmVzc2'
    'lvblIFcmlnaHQ=');

@$core.Deprecated('Use callExpressionDescriptor instead')
const CallExpression$json = {
  '1': 'CallExpression',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'arguments',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'arguments'
    },
  ],
};

/// Descriptor for `CallExpression`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List callExpressionDescriptor = $convert.base64Decode(
    'Cg5DYWxsRXhwcmVzc2lvbhISCgRuYW1lGAEgASgJUgRuYW1lEjgKCWFyZ3VtZW50cxgCIAMoCz'
    'IaLnN0b3J5YnVuZGxlLnYxLkV4cHJlc3Npb25SCWFyZ3VtZW50cw==');

@$core.Deprecated('Use listExpressionDescriptor instead')
const ListExpression$json = {
  '1': 'ListExpression',
  '2': [
    {
      '1': 'items',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.storybundle.v1.Expression',
      '10': 'items'
    },
  ],
};

/// Descriptor for `ListExpression`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listExpressionDescriptor = $convert.base64Decode(
    'Cg5MaXN0RXhwcmVzc2lvbhIwCgVpdGVtcxgBIAMoCzIaLnN0b3J5YnVuZGxlLnYxLkV4cHJlc3'
    'Npb25SBWl0ZW1z');
