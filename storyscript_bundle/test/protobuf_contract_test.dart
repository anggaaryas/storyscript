import 'package:flutter_test/flutter_test.dart';
import 'package:fixnum/fixnum.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

void main() {
  test('generated Dart types preserve recursive and oneof semantics', () {
    final story = CompiledStory(
      formatVersion: 1,
      project: ProjectMetadata(id: 'p', name: 'Project', version: '1.0.0'),
      initialization: Initialization(
        startScene: 'start',
        variables: <VariableDefinition>[
          VariableDefinition(
            name: 'score',
            type: VariableType.VARIABLE_TYPE_INTEGER,
            value: Expression(integer: Int64(7)),
          ),
        ],
      ),
      scenes: <Scene>[
        Scene(
          label: 'start',
          story: StoryBlock(
            statements: <StoryStatement>[
              StoryStatement(
                narration: Narration(
                  text: InterpolatedString(
                    segments: <StringSegment>[
                      StringSegment(literal: 'Score: '),
                      StringSegment(variable: 'score'),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );

    final decoded = CompiledStory.fromBuffer(story.writeToBuffer());

    expect(decoded.initialization.variables.single.value.integer.toInt(), 7);
    expect(
      decoded
          .scenes
          .single
          .story
          .statements
          .single
          .narration
          .text
          .segments
          .last
          .variable,
      'score',
    );
    expect(
      storyBundleSchemaSha256,
      'aee45aee882cacdfeb6d320808ed0ac72805da27c87b5486289e6a9b6be9d70a',
    );
  });
}
