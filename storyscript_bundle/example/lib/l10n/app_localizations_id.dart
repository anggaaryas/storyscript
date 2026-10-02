// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Indonesian (`id`).
class AppLocalizationsId extends AppLocalizations {
  AppLocalizationsId([String locale = 'id']) : super(locale);

  @override
  String get appTitle => 'Contoh StoryScript';

  @override
  String get gameTitle => 'Sinyal di Station Nine';

  @override
  String get inspectorTitle => 'Inspektur StoryBundle';

  @override
  String get openInspector => 'Buka demo Inspektur Bundle';

  @override
  String get language => 'Bahasa';

  @override
  String get english => 'Inggris';

  @override
  String get indonesian => 'Indonesia';

  @override
  String get switchingLanguage => 'Mengganti bahasa…';

  @override
  String switchFailed(Object details) {
    return 'Tidak dapat mengganti bahasa: $details';
  }

  @override
  String localeStatus(Object requested, Object resolved) {
    return 'Diminta: $requested • Cerita: $resolved';
  }

  @override
  String get noLocale => 'Belum dipilih';

  @override
  String get loading => 'Memuat…';

  @override
  String get gameStartFailed =>
      'Tidak dapat memverifikasi atau memulai permainan.';

  @override
  String get unknownError => 'Kesalahan tidak diketahui';

  @override
  String get retry => 'Coba lagi';

  @override
  String get chapter => 'BAB SATU / FAJAR PINJAMAN';

  @override
  String get backgroundLabel => 'Latar Station Nine';

  @override
  String portraitLabel(Object actor) {
    return 'Potret $actor';
  }

  @override
  String get speaker => 'Pembicara';

  @override
  String artworkUnavailable(Object details) {
    return 'Ilustrasi tidak tersedia: $details';
  }

  @override
  String gameError(Object details) {
    return 'Kesalahan permainan: $details';
  }

  @override
  String choiceLabel(Object number, Object text) {
    return 'Pilihan $number: $text';
  }

  @override
  String get playAgain => 'Main lagi';

  @override
  String get continueAction => 'Lanjutkan';

  @override
  String get chooseHint => 'Pilih kelanjutan cerita.';

  @override
  String get continueHint => 'Lanjutkan untuk membuka halaman berikutnya.';

  @override
  String get restartHint => 'Kamu dapat memulai bab ini lagi.';

  @override
  String enteringScene(Object scene) {
    return 'Memasuki $scene…';
  }

  @override
  String get station => 'stasiun';

  @override
  String get choicePrompt => 'Apa yang akan kamu lakukan?';

  @override
  String get theEnd => 'Tamat';

  @override
  String get gameStopped => 'Permainan berhenti';

  @override
  String get mediaCue => 'Isyarat media';

  @override
  String get loadSignedBundle => 'Muat contoh StoryBundle bertanda tangan';

  @override
  String get reload => 'Muat ulang';

  @override
  String get load => 'Muat';

  @override
  String get release => 'Lepaskan';

  @override
  String get noBundle => 'Belum ada bundle yang dimuat';

  @override
  String get unsignedBundle => 'Bundle pengembangan tanpa tanda tangan';

  @override
  String get trustedSignature => 'Tanda tangan dipercaya dan terverifikasi';

  @override
  String get loadFailed => 'Gagal memuat';

  @override
  String get bundleDisposed => 'Bundle dilepaskan';

  @override
  String get verifyingBundle => 'Memverifikasi StoryBundle';

  @override
  String get resourceReleased => 'Sumber daya Rust telah dilepaskan.';

  @override
  String get loadToInspect =>
      'Muat contoh bertanda tangan untuk memeriksa model terverifikasinya.';

  @override
  String assetsHeading(Object count) {
    return 'Aset ($count)';
  }

  @override
  String assetBytes(Object hash, Object size) {
    return '$size bita • $hash';
  }

  @override
  String previewAsset(Object path) {
    return 'Pratinjau $path';
  }

  @override
  String get previewFailed => 'Pratinjau aset gagal';

  @override
  String assetDecodeFailed(Object details) {
    return 'Penguraian aset gagal: $details';
  }

  @override
  String get verifiedMetadata => 'Metadata terverifikasi';

  @override
  String get project => 'Proyek';

  @override
  String get projectId => 'ID proyek';

  @override
  String get projectVersion => 'Versi proyek';

  @override
  String get format => 'Format';

  @override
  String get compiler => 'Kompiler';

  @override
  String get schema => 'Skema';

  @override
  String get signer => 'Penanda tangan';

  @override
  String get defaultLocale => 'Bahasa bawaan';

  @override
  String get supportedLocaleHeading => 'Bahasa yang didukung';

  @override
  String get notLocalized => 'Tidak dilokalkan';

  @override
  String get semanticModel => 'Model semantik';

  @override
  String get summary => 'Ringkasan';

  @override
  String modelSummary(Object actors, Object logic, Object scenes) {
    return '$scenes adegan • $logic blok logika • $actors aktor';
  }

  @override
  String scenesHeading(Object count) {
    return 'Adegan ($count)';
  }

  @override
  String actorsHeading(Object count) {
    return 'Aktor ($count)';
  }

  @override
  String logicHeading(Object count) {
    return 'Logika ($count)';
  }

  @override
  String storyStatements(Object count) {
    return '$count pernyataan cerita';
  }

  @override
  String portraits(Object count) {
    return '$count potret';
  }

  @override
  String statements(Object count) {
    return '$count pernyataan';
  }
}
