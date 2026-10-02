import 'package:flutter_test/flutter_test.dart';
import 'package:fixnum/fixnum.dart';
import 'package:storyscript_bundle/storyscript_bundle.dart';

void main() {
  test('locale metadata and typed message unions exclude catalog bodies', () {
    final story = CompiledStory(
      localization: LocalizationMetadata(
        defaultLocale: 'en',
        supportedLocales: ['en', 'id'],
      ),
      scenes: [
        Scene(
          story: StoryBlock(
            statements: [
              StoryStatement(
                narration: Narration(
                  text: StoryText(
                    message: MessageReference(
                      id: 'greeting',
                      arguments: [
                        MessageArgument(
                          name: 'name',
                          type: VariableType.VARIABLE_TYPE_STRING,
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
    final decoded = CompiledStory.fromBuffer(story.writeToBuffer());
    expect(decoded.localization.defaultLocale, 'en');
    expect(decoded.localization.supportedLocales, ['en', 'id']);
    final text = decoded.scenes.single.story.statements.single.narration.text;
    expect(text.whichValue(), StoryText_Value.message);
    expect(text.message.id, 'greeting');
    expect(text.message.arguments.single.name, 'name');
    expect(text.hasPlain(), isFalse);
  });
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
                  text: StoryText(
                    plain: InterpolatedString(
                      segments: <StringSegment>[
                        StringSegment(literal: 'Score: '),
                        StringSegment(variable: 'score'),
                      ],
                    ),
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
          .plain
          .segments
          .last
          .variable,
      'score',
    );
    expect(
      storyBundleSchemaSha256,
      '0c1bacf81cbe7b4b2cafb68c7d1305f383efda9b3548e7ebfd2b0a5f158d2810',
    );
  });
}
