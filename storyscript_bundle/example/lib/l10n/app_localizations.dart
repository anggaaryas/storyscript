import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_id.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('id'),
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'StoryScript Example'**
  String get appTitle;

  /// No description provided for @gameTitle.
  ///
  /// In en, this message translates to:
  /// **'Signal at Station Nine'**
  String get gameTitle;

  /// No description provided for @inspectorTitle.
  ///
  /// In en, this message translates to:
  /// **'StoryBundle Inspector'**
  String get inspectorTitle;

  /// No description provided for @openInspector.
  ///
  /// In en, this message translates to:
  /// **'Open demo Bundle Inspector'**
  String get openInspector;

  /// No description provided for @language.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get language;

  /// No description provided for @english.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get english;

  /// No description provided for @indonesian.
  ///
  /// In en, this message translates to:
  /// **'Indonesian'**
  String get indonesian;

  /// No description provided for @switchingLanguage.
  ///
  /// In en, this message translates to:
  /// **'Changing language…'**
  String get switchingLanguage;

  /// No description provided for @switchFailed.
  ///
  /// In en, this message translates to:
  /// **'Could not change language: {details}'**
  String switchFailed(Object details);

  /// No description provided for @localeStatus.
  ///
  /// In en, this message translates to:
  /// **'Requested: {requested} • Story: {resolved}'**
  String localeStatus(Object requested, Object resolved);

  /// No description provided for @noLocale.
  ///
  /// In en, this message translates to:
  /// **'Not selected'**
  String get noLocale;

  /// No description provided for @loading.
  ///
  /// In en, this message translates to:
  /// **'Loading…'**
  String get loading;

  /// No description provided for @gameStartFailed.
  ///
  /// In en, this message translates to:
  /// **'Could not verify or start the game.'**
  String get gameStartFailed;

  /// No description provided for @unknownError.
  ///
  /// In en, this message translates to:
  /// **'Unknown error'**
  String get unknownError;

  /// No description provided for @retry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get retry;

  /// No description provided for @chapter.
  ///
  /// In en, this message translates to:
  /// **'CHAPTER ONE / A BORROWED DAWN'**
  String get chapter;

  /// No description provided for @backgroundLabel.
  ///
  /// In en, this message translates to:
  /// **'Station Nine background'**
  String get backgroundLabel;

  /// No description provided for @portraitLabel.
  ///
  /// In en, this message translates to:
  /// **'{actor} portrait'**
  String portraitLabel(Object actor);

  /// No description provided for @speaker.
  ///
  /// In en, this message translates to:
  /// **'Speaker'**
  String get speaker;

  /// No description provided for @artworkUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Artwork unavailable: {details}'**
  String artworkUnavailable(Object details);

  /// No description provided for @gameError.
  ///
  /// In en, this message translates to:
  /// **'Game error: {details}'**
  String gameError(Object details);

  /// No description provided for @choiceLabel.
  ///
  /// In en, this message translates to:
  /// **'Option {number}: {text}'**
  String choiceLabel(Object number, Object text);

  /// No description provided for @playAgain.
  ///
  /// In en, this message translates to:
  /// **'Play again'**
  String get playAgain;

  /// No description provided for @continueAction.
  ///
  /// In en, this message translates to:
  /// **'Continue'**
  String get continueAction;

  /// No description provided for @chooseHint.
  ///
  /// In en, this message translates to:
  /// **'Choose how the story continues.'**
  String get chooseHint;

  /// No description provided for @continueHint.
  ///
  /// In en, this message translates to:
  /// **'Continue to turn the page.'**
  String get continueHint;

  /// No description provided for @restartHint.
  ///
  /// In en, this message translates to:
  /// **'You can begin the chapter again.'**
  String get restartHint;

  /// No description provided for @enteringScene.
  ///
  /// In en, this message translates to:
  /// **'Entering {scene}…'**
  String enteringScene(Object scene);

  /// No description provided for @station.
  ///
  /// In en, this message translates to:
  /// **'the station'**
  String get station;

  /// No description provided for @choicePrompt.
  ///
  /// In en, this message translates to:
  /// **'What will you do?'**
  String get choicePrompt;

  /// No description provided for @theEnd.
  ///
  /// In en, this message translates to:
  /// **'The End'**
  String get theEnd;

  /// No description provided for @gameStopped.
  ///
  /// In en, this message translates to:
  /// **'Game stopped'**
  String get gameStopped;

  /// No description provided for @mediaCue.
  ///
  /// In en, this message translates to:
  /// **'Media cue'**
  String get mediaCue;

  /// No description provided for @loadSignedBundle.
  ///
  /// In en, this message translates to:
  /// **'Load signed StoryBundle fixture'**
  String get loadSignedBundle;

  /// No description provided for @reload.
  ///
  /// In en, this message translates to:
  /// **'Reload'**
  String get reload;

  /// No description provided for @load.
  ///
  /// In en, this message translates to:
  /// **'Load'**
  String get load;

  /// No description provided for @release.
  ///
  /// In en, this message translates to:
  /// **'Release'**
  String get release;

  /// No description provided for @noBundle.
  ///
  /// In en, this message translates to:
  /// **'No bundle loaded'**
  String get noBundle;

  /// No description provided for @unsignedBundle.
  ///
  /// In en, this message translates to:
  /// **'Unsigned development bundle'**
  String get unsignedBundle;

  /// No description provided for @trustedSignature.
  ///
  /// In en, this message translates to:
  /// **'Signature trusted and verified'**
  String get trustedSignature;

  /// No description provided for @loadFailed.
  ///
  /// In en, this message translates to:
  /// **'Load failed'**
  String get loadFailed;

  /// No description provided for @bundleDisposed.
  ///
  /// In en, this message translates to:
  /// **'Bundle disposed'**
  String get bundleDisposed;

  /// No description provided for @verifyingBundle.
  ///
  /// In en, this message translates to:
  /// **'Verifying StoryBundle'**
  String get verifyingBundle;

  /// No description provided for @resourceReleased.
  ///
  /// In en, this message translates to:
  /// **'The Rust resource has been released.'**
  String get resourceReleased;

  /// No description provided for @loadToInspect.
  ///
  /// In en, this message translates to:
  /// **'Load the signed fixture to inspect its verified model.'**
  String get loadToInspect;

  /// No description provided for @assetsHeading.
  ///
  /// In en, this message translates to:
  /// **'Assets ({count})'**
  String assetsHeading(Object count);

  /// No description provided for @assetBytes.
  ///
  /// In en, this message translates to:
  /// **'{size} bytes • {hash}'**
  String assetBytes(Object hash, Object size);

  /// No description provided for @previewAsset.
  ///
  /// In en, this message translates to:
  /// **'Preview {path}'**
  String previewAsset(Object path);

  /// No description provided for @previewFailed.
  ///
  /// In en, this message translates to:
  /// **'Asset preview failed'**
  String get previewFailed;

  /// No description provided for @assetDecodeFailed.
  ///
  /// In en, this message translates to:
  /// **'Asset decode failed: {details}'**
  String assetDecodeFailed(Object details);

  /// No description provided for @verifiedMetadata.
  ///
  /// In en, this message translates to:
  /// **'Verified metadata'**
  String get verifiedMetadata;

  /// No description provided for @project.
  ///
  /// In en, this message translates to:
  /// **'Project'**
  String get project;

  /// No description provided for @projectId.
  ///
  /// In en, this message translates to:
  /// **'Project ID'**
  String get projectId;

  /// No description provided for @projectVersion.
  ///
  /// In en, this message translates to:
  /// **'Project version'**
  String get projectVersion;

  /// No description provided for @format.
  ///
  /// In en, this message translates to:
  /// **'Format'**
  String get format;

  /// No description provided for @compiler.
  ///
  /// In en, this message translates to:
  /// **'Compiler'**
  String get compiler;

  /// No description provided for @schema.
  ///
  /// In en, this message translates to:
  /// **'Schema'**
  String get schema;

  /// No description provided for @signer.
  ///
  /// In en, this message translates to:
  /// **'Signer'**
  String get signer;

  /// No description provided for @defaultLocale.
  ///
  /// In en, this message translates to:
  /// **'Default locale'**
  String get defaultLocale;

  /// No description provided for @supportedLocaleHeading.
  ///
  /// In en, this message translates to:
  /// **'Supported locales'**
  String get supportedLocaleHeading;

  /// No description provided for @notLocalized.
  ///
  /// In en, this message translates to:
  /// **'Not localized'**
  String get notLocalized;

  /// No description provided for @semanticModel.
  ///
  /// In en, this message translates to:
  /// **'Semantic model'**
  String get semanticModel;

  /// No description provided for @summary.
  ///
  /// In en, this message translates to:
  /// **'Summary'**
  String get summary;

  /// No description provided for @modelSummary.
  ///
  /// In en, this message translates to:
  /// **'{scenes} scenes • {logic} logic blocks • {actors} actors'**
  String modelSummary(Object actors, Object logic, Object scenes);

  /// No description provided for @scenesHeading.
  ///
  /// In en, this message translates to:
  /// **'Scenes ({count})'**
  String scenesHeading(Object count);

  /// No description provided for @actorsHeading.
  ///
  /// In en, this message translates to:
  /// **'Actors ({count})'**
  String actorsHeading(Object count);

  /// No description provided for @logicHeading.
  ///
  /// In en, this message translates to:
  /// **'Logic ({count})'**
  String logicHeading(Object count);

  /// No description provided for @storyStatements.
  ///
  /// In en, this message translates to:
  /// **'{count} story statements'**
  String storyStatements(Object count);

  /// No description provided for @portraits.
  ///
  /// In en, this message translates to:
  /// **'{count} portraits'**
  String portraits(Object count);

  /// No description provided for @statements.
  ///
  /// In en, this message translates to:
  /// **'{count} statements'**
  String statements(Object count);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'id'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'id':
      return AppLocalizationsId();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
