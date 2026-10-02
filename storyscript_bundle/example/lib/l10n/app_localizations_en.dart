// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'StoryScript Example';

  @override
  String get gameTitle => 'Signal at Station Nine';

  @override
  String get inspectorTitle => 'StoryBundle Inspector';

  @override
  String get openInspector => 'Open demo Bundle Inspector';

  @override
  String get language => 'Language';

  @override
  String get english => 'English';

  @override
  String get indonesian => 'Indonesian';

  @override
  String get switchingLanguage => 'Changing language…';

  @override
  String switchFailed(Object details) {
    return 'Could not change language: $details';
  }

  @override
  String localeStatus(Object requested, Object resolved) {
    return 'Requested: $requested • Story: $resolved';
  }

  @override
  String get noLocale => 'Not selected';

  @override
  String get loading => 'Loading…';

  @override
  String get gameStartFailed => 'Could not verify or start the game.';

  @override
  String get unknownError => 'Unknown error';

  @override
  String get retry => 'Retry';

  @override
  String get chapter => 'CHAPTER ONE / A BORROWED DAWN';

  @override
  String get backgroundLabel => 'Station Nine background';

  @override
  String portraitLabel(Object actor) {
    return '$actor portrait';
  }

  @override
  String get speaker => 'Speaker';

  @override
  String artworkUnavailable(Object details) {
    return 'Artwork unavailable: $details';
  }

  @override
  String gameError(Object details) {
    return 'Game error: $details';
  }

  @override
  String choiceLabel(Object number, Object text) {
    return 'Option $number: $text';
  }

  @override
  String get playAgain => 'Play again';

  @override
  String get continueAction => 'Continue';

  @override
  String get chooseHint => 'Choose how the story continues.';

  @override
  String get continueHint => 'Continue to turn the page.';

  @override
  String get restartHint => 'You can begin the chapter again.';

  @override
  String enteringScene(Object scene) {
    return 'Entering $scene…';
  }

  @override
  String get station => 'the station';

  @override
  String get choicePrompt => 'What will you do?';

  @override
  String get theEnd => 'The End';

  @override
  String get gameStopped => 'Game stopped';

  @override
  String get mediaCue => 'Media cue';

  @override
  String get loadSignedBundle => 'Load signed StoryBundle fixture';

  @override
  String get reload => 'Reload';

  @override
  String get load => 'Load';

  @override
  String get release => 'Release';

  @override
  String get noBundle => 'No bundle loaded';

  @override
  String get unsignedBundle => 'Unsigned development bundle';

  @override
  String get trustedSignature => 'Signature trusted and verified';

  @override
  String get loadFailed => 'Load failed';

  @override
  String get bundleDisposed => 'Bundle disposed';

  @override
  String get verifyingBundle => 'Verifying StoryBundle';

  @override
  String get resourceReleased => 'The Rust resource has been released.';

  @override
  String get loadToInspect =>
      'Load the signed fixture to inspect its verified model.';

  @override
  String assetsHeading(Object count) {
    return 'Assets ($count)';
  }

  @override
  String assetBytes(Object hash, Object size) {
    return '$size bytes • $hash';
  }

  @override
  String previewAsset(Object path) {
    return 'Preview $path';
  }

  @override
  String get previewFailed => 'Asset preview failed';

  @override
  String assetDecodeFailed(Object details) {
    return 'Asset decode failed: $details';
  }

  @override
  String get verifiedMetadata => 'Verified metadata';

  @override
  String get project => 'Project';

  @override
  String get projectId => 'Project ID';

  @override
  String get projectVersion => 'Project version';

  @override
  String get format => 'Format';

  @override
  String get compiler => 'Compiler';

  @override
  String get schema => 'Schema';

  @override
  String get signer => 'Signer';

  @override
  String get defaultLocale => 'Default locale';

  @override
  String get supportedLocaleHeading => 'Supported locales';

  @override
  String get notLocalized => 'Not localized';

  @override
  String get semanticModel => 'Semantic model';

  @override
  String get summary => 'Summary';

  @override
  String modelSummary(Object actors, Object logic, Object scenes) {
    return '$scenes scenes • $logic logic blocks • $actors actors';
  }

  @override
  String scenesHeading(Object count) {
    return 'Scenes ($count)';
  }

  @override
  String actorsHeading(Object count) {
    return 'Actors ($count)';
  }

  @override
  String logicHeading(Object count) {
    return 'Logic ($count)';
  }

  @override
  String storyStatements(Object count) {
    return '$count story statements';
  }

  @override
  String portraits(Object count) {
    return '$count portraits';
  }

  @override
  String statements(Object count) {
    return '$count statements';
  }
}
