# Author comment removed from bundles.
-brand = StoryScript
greeting = Hello { $name } from { -brand }.
count-line = { $count ->
    [one] One item
   *[other] { NUMBER($count) } items
    }
continue-choice = { $ready ->
    [true] Continue
   *[false] Wait
    }
