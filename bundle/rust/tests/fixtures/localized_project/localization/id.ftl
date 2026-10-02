# Komentar penerjemah tidak disertakan dalam bundle.
-brand = StoryScript
greeting = Halo { $name } dari { -brand }.
count-line = { $count ->
    [one] Satu barang
   *[other] { NUMBER($count) } barang
    }
continue-choice = { $ready ->
    [true] Lanjutkan
   *[false] Tunggu
    }
